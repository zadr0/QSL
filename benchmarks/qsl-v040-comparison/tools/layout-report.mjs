import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const directory = path.join(root, 'results/layout');
const read = async filename => JSON.parse((await fs.readFile(filename, 'utf8')).replace(/^\uFEFF/, ''));
const sha256 = async filename => createHash('sha256').update(await fs.readFile(filename)).digest('hex').toUpperCase();
const environment = await read(path.join(directory, 'environment.json'));
const runs = await read(path.join(directory, 'runs.json'));
const probe = await read(path.join(directory, 'profiles.json'));
assert.equal(runs.length, 2);
assert(runs.every(r => r.exitCode === 0));
assert.equal(await sha256(environment.executable), environment.executableSha256);
for (const [relative, expected] of Object.entries(environment.sources)) {
  assert.equal(await sha256(path.resolve(root, 'layout', relative)), expected, relative);
}

const samples = [];
for (const run of runs) {
  const folder = path.join(directory, run.run, 'criterion');
  const parsed = new Map();
  for (const entry of await fs.readdir(folder, {withFileTypes: true})) {
    if (!entry.isDirectory()) continue;
    const result = path.join(folder, entry.name, 'new');
    const benchmark = await read(path.join(result, 'benchmark.json'));
    const estimates = await read(path.join(result, 'estimates.json'));
    parsed.set(benchmark.full_id, estimates.median);
  }
  assert.equal(parsed.size, 16);
  samples.push(parsed);
}

const variants = ['declared', 'auto', 'auto_wire_1', 'manual'];
const operations = ['write', 'read', 'scan_hot', 'scan_large'];
const profiles = variants.map(name => {
  const raw = probe.profiles.find(profile => profile.case === name);
  assert(raw);
  const wireOffsets = ['11', '2233445566778899', 'aabb', 'ccddeeff', '10'].map(marker => {
    const position = raw.frame_example_hex.match(new RegExp(marker, 'i'))?.index;
    assert(position != null && position % 2 === 0);
    return position / 2;
  });
  return {...raw, wire_offsets: wireOffsets, padding_bytes: raw.size - raw.frame_bytes};
});
assert.equal(profiles[0].frame_example_hex, profiles[1].frame_example_hex);
const measurements = [];
for (const operation of operations) {
  const records = operation === 'scan_large' ? probe.large_count : operation === 'scan_hot' ? probe.fixture_count : 1;
  const baselineKey = operation + '/declared';
  const baseline = (samples[0].get(baselineKey).point_estimate + samples[1].get(baselineKey).point_estimate) / 2;
  for (const variant of variants) {
    const id = operation + '/' + variant;
    const [a, b] = samples.map(map => map.get(id));
    assert(a && b, id);
    const median = (a.point_estimate + b.point_estimate) / 2;
    measurements.push({
      id, variant, operation, records,
      median1_ns: a.point_estimate, median2_ns: b.point_estimate, mean_of_medians_ns: median,
      run1_ci99_low: a.confidence_interval.lower_bound, run1_ci99_high: a.confidence_interval.upper_bound,
      run2_ci99_low: b.confidence_interval.lower_bound, run2_ci99_high: b.confidence_interval.upper_bound,
      ns_per_record: median / records, speed_index: 100 * baseline / median,
      drift_percent: 100 * Math.abs(a.point_estimate - b.point_estimate) / median,
    });
  }
}
const columns = Object.keys(measurements[0]);
const csv = columns.join(',') + '\n' + measurements.map(row => columns.map(key => row[key]).join(',')).join('\n') + '\n';
await fs.writeFile(path.join(directory, 'measurements.csv'), csv);
await fs.writeFile(path.join(directory, 'summary.json'), JSON.stringify({
  environment, runs, measurements, profiles, sourceIntegrity: 'unchanged',
}, null, 2) + '\n');

const nf = number => number.toFixed(2);
const labels = {declared: 'Порядок объявления', auto: 'auto', auto_wire_1: 'auto[wire=1]', manual: 'Ручная перестановка'};
let doc = '# QSL 0.4.0 — автоматическая раскладка, память и скорость\n\n';
doc += 'Четыре схемы содержат одни поля с одними значениями: `flag:u8`, `count:u64`, `id:u16`, `time:u32`, `quality:u8`. Сравниваются порядок объявления, `auto`, `auto[wire=1]` и ручная расстановка по убывающему выравниванию. Входы и потребитель одинаковы. [Схемы](layout/schemas/layout.qsl)\n\n';
doc += '## Память объекта и кадр\n\n';
doc += 'Смещения измерены в байтах, в порядке **flag, count, id, time, quality**. Размер объекта включает выравнивающие промежутки. Индекс памяти — отношение размера объекта к исходному, где исходный размер равен 100. Объём большого массива рассчитан для '+probe.large_count+' объектов, включая промежутки между полями.\n\n';
doc += '| Схема | Rust, B | Выравнивание, B | Промежутки, B | Индекс памяти | Большой массив, MiB | Смещения Rust | Кадр, B | Смещения кадра |\n';
doc += '|---|---:|---:|---:|---:|---:|---|---:|---|\n';
for (const p of profiles) {
  doc += '| '+labels[p.case]+' | '+p.size+' | '+p.alignment+' | '+p.padding_bytes+' | '+nf(100*p.size/profiles[0].size)+' | '+nf(p.size*probe.large_count/2**20)+' | '+p.offsets.join(', ')+' | '+p.frame_bytes+' | '+p.wire_offsets.join(', ')+' |\n';
}
doc += '\n`auto` сохранил кадр побайтно на всех '+probe.fixture_count+' записях. В этом наборе `auto[wire=1]` '+(profiles[2].frame_example_hex===profiles[0].frame_example_hex?'сохранил':'изменил')+' порядок полей в кадре. Ручная перестановка меняет порядок объявления и байты кадра. Все варианты восстановили исходные значения. Отдельный счётчик аллокаций за 256 операций проверяемой записи и чтения показал '+profiles.map(p=>labels[p.case]+' '+p.write_allocs_256+'/'+p.read_allocs_256).join(', ')+'.\n\n';
doc += '## Скорость\n\n';
doc += 'Время приведено в нс на объект. Запись работает с заранее выделенным буфером. Чтение создаёт Rust-объект и вычисляет контрольную сумму. Проходы читают все поля в непрерывном массиве из '+probe.fixture_count+' или '+probe.large_count+' объектов и суммируют тот же контрольный код. Индекс скорости равен 100 × время исходной схемы / время варианта. Значение выше 100 соответствует меньшему времени.\n\n';
doc += '| Схема | Запись, нс | Индекс | Чтение, нс | Индекс | Малый проход, нс | Индекс | Большой проход, нс | Индекс |\n';
doc += '|---|---:|---:|---:|---:|---:|---:|---:|---:|\n';
for (const variant of variants) {
  const rows = measurements.filter(row => row.variant === variant);
  doc += '| '+labels[variant]+' | '+rows.map(row=>nf(row.ns_per_record)+' | '+nf(row.speed_index)).join(' | ')+' |\n';
}
doc += '\nМетодика Criterion: 16 случаев в каждом из двух проходов в противоположном порядке. 200 выборок на случай, 1 с прогрева, 3 с целевого измерения, по 10 000 bootstrap-повторов и CI 99%. В таблице средние двух медиан. CI большого и малого проходов в CSV относятся ко всему массиву, а `ns_per_record` приведено к одному объекту. Фон отдельно от процесса: '+runs.map(r=>nf(r.otherSystemCpuCoreEquivalent)).join(' / ')+' логического ядра. Случаев с дрейфом выше 10%: '+measurements.filter(r=>r.drift_percent>10).length+'. Мелкие отличия сверяйте с исходными CI и дрейфом.\n\n';
doc += '[CSV и CI](results/layout/measurements.csv) · [метаданные](results/layout/environment.json) · [смещения, кадры и аллокации](results/layout/profiles.json) · [сырые данные](results/layout)\n';
await fs.writeFile(path.join(root, 'LAYOUT.md'), doc);
console.log('Layout report verified:', measurements.length, 'cases across', runs.length, 'passes');
