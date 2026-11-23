import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const read = async file => {
  const bytes = await fs.readFile(path.join(root, file));
  const encoding = bytes[0] === 0xff && bytes[1] === 0xfe ? 'utf16le' : 'utf8';
  return JSON.parse(bytes.toString(encoding).replace(/^\uFEFF/, ''));
};
const hash = async file => createHash('sha256').update(await fs.readFile(file)).digest('hex').toUpperCase();
const save = (file, content) => fs.writeFile(path.join(root, file), content + '\n', 'utf8');
const avg = values => values.reduce((sum, value) => sum + value, 0) / values.length;
const drift = values => 100 * (Math.max(...values) - Math.min(...values)) / avg(values);
const names = {
  qsl:'QSL', qsl_bitmap:'QSL bitmap', protobuf:'Protobuf / Prost', bincode_varint:'Bincode varint',
  capnp:'Cap’n Proto', capnp_packed:'Cap’n Proto packed',
  string:'String', stack_string:'StackString', qsl_lz:'QSL-LZ / InlineString',
  fields:'Чтение по полям', copy_aligned:'Трансмутация с копией', copy_unaligned:'Копия со смещённого адреса', overlay_aligned:'Заимствование overlay',
};
const codecs = ['qsl','qsl_bitmap','protobuf','bincode_varint','capnp','capnp_packed'];
const workloads = {api:'API · вложенные сообщения', heartbeat:'Heartbeat · контроль связи', event:'Event · одно событие'};
const corpora = {periodic_16:'Период 16 байт', structured_log:'Структурированный журнал', noisy_ascii:'Шумовой ASCII'};

async function criterion(suite, expected) {
  const folder = 'results/' + suite;
  const environment = await read(folder + '/environment.json');
  const runs = await read(folder + '/runs.json');
  assert.equal(runs.length, 2);
  assert(runs.every(run => run.exitCode === 0));
  assert.equal(await hash(path.resolve(root, environment.executable)), environment.executableSha256);
  for (const [filename, digest] of Object.entries(environment.sources)) {
    assert.equal(await hash(path.resolve(root, suite, filename.replaceAll('\\', '/'))), digest, filename);
  }
  const maps = [];
  for (const run of runs) {
    const base = folder + '/' + run.run + '/criterion';
    const collected = new Map();
    for (const entry of await fs.readdir(path.join(root, base), {withFileTypes:true})) {
      if (!entry.isDirectory()) continue;
      const dir = base + '/' + entry.name + '/new/';
      const benchmark = await read(dir + 'benchmark.json');
      const estimates = await read(dir + 'estimates.json');
      assert(!collected.has(benchmark.full_id), 'Duplicate benchmark ' + benchmark.full_id);
      collected.set(benchmark.full_id, estimates.median);
    }
    assert.equal(collected.size, expected);
    maps.push(collected);
  }
  const rows = [...maps[0]].map(([id, first]) => {
    const second = maps[1].get(id);
    assert(second, 'Missing repeat ' + id);
    const [workload, operation, codec] = id.split('/');
    const ns = [first.point_estimate, second.point_estimate];
    return {id, workload, operation, codec, median_ns:avg(ns), runs_ns:ns,
      ci99_ns:[first, second].map(p => [p.confidence_interval.lower_bound, p.confidence_interval.upper_bound]),
      drift_percent:drift(ns)};
  });
  return {environment, runs, rows};
}

function profiles(raw, timing) {
  return raw.profiles.map(({workload, codec, object_bytes, samples}) => {
    assert.equal(samples.length, 256);
    const encode = timing.rows.find(x => x.workload === workload && x.codec === codec && x.operation === 'encode');
    const decode = timing.rows.find(x => x.workload === workload && x.codec === codec && x.operation === 'decode');
    assert(encode && decode);
    const wire = samples.map(s => s.wire_bytes);
    return {workload, codec, object_bytes:object_bytes ?? raw.owned_inline_bytes,
      wire_mean:avg(wire), wire_min:Math.min(...wire), wire_max:Math.max(...wire),
      encode_ns:encode.median_ns, decode_ns:decode.median_ns,
      drift_percent:Math.max(encode.drift_percent, decode.drift_percent),
      encode_allocs:avg(samples.map(s=>s.encode_alloc_calls)), encode_alloc_bytes:avg(samples.map(s=>s.encode_alloc_bytes)),
      decode_allocs:avg(samples.map(s=>s.decode_alloc_calls)), decode_alloc_bytes:avg(samples.map(s=>s.decode_alloc_bytes)),
    };
  });
}

const realistic = await criterion('realistic', 36);
const features = await criterion('qsl-features', 22);
const transport = await read('results/realistic/transport-environment.json');
assert.equal(await hash(path.join(root, 'realistic/k6/api.js')), transport.k6ScriptSha256);
assert.equal(await hash(path.join(root, 'target/release/socket.exe')), transport.socketExecutableSha256);
assert.equal(await hash(path.join(root, 'target/release/api.exe')), transport.serverExecutableSha256);
const codecRows = profiles(await read('results/realistic/profiles.json'), realistic);
const featureProbe = await read('results/qsl-features/profiles.json');
const stringRows = profiles(featureProbe, features);
const native = features.rows.filter(row=>row.workload === 'native');
assert.equal(codecRows.length, 18);
assert.equal(stringRows.length, 9);
assert.equal(native.length, 4);
const socketSamples = await read('results/realistic/socket.json');
const httpRuns = await read('results/realistic/http-runs.json');
const fixtures = await read('results/realistic/fixtures/manifest.json');
assert.equal(socketSamples.length, 12);
assert.equal(httpRuns.length, 12);
const httpRows = [];
const tcpRows = [];
function percentile(samples, fraction) {
  const sorted = samples.toSorted((a, b)=>a-b);
  const position = (sorted.length - 1) * fraction;
  return sorted[Math.floor(position)] + (position % 1) * (sorted[Math.ceil(position)] - sorted[Math.floor(position)]);
}
for (const codec of codecs) {
  const sessions = socketSamples.filter(run => run.codec === codec);
  const http = httpRuns.filter(run => run.codec === codec);
  assert.deepEqual(sessions.map(run=>run.pass).sort(), [1,2]);
  assert.deepEqual(http.map(run=>run.pass).sort(), [1,2]);
  for (const run of sessions) {
    assert.equal(run.measured_rounds, 2048);
    assert.equal(run.batch_size, 32);
    assert.equal(run.heartbeat_rtt_ns.length, run.measured_rounds);
    assert.equal(run.event_batch_rtt_ns.length, run.measured_rounds);
  }
  const tcpRaw = sessions.map(run => {
    const count = run.measured_rounds;
    return {
      pass:run.pass, count, batch:run.batch_size,
      heartbeat_p50_us:percentile(run.heartbeat_rtt_ns, .5)/1000,
      heartbeat_p95_us:percentile(run.heartbeat_rtt_ns, .95)/1000,
      heartbeat_p99_us:percentile(run.heartbeat_rtt_ns, .99)/1000,
      batch_p50_us:percentile(run.event_batch_rtt_ns, .5)/1000,
      batch_p95_us:percentile(run.event_batch_rtt_ns, .95)/1000,
      batch_p99_us:percentile(run.event_batch_rtt_ns, .99)/1000,
      heartbeat_bytes:run.heartbeat_two_way_framed_bytes/count,
      event_bytes:(run.events_framed_bytes+run.ack_framed_bytes)/(count*run.batch_size),
      active_events_per_s:1e9*count*run.batch_size/run.event_batch_rtt_ns.reduce((sum,n)=>sum+n,0),
    };
  });
  tcpRows.push({codec, trials:tcpRaw,
    ...Object.fromEntries(Object.keys(tcpRaw[0]).filter(k=>!['pass','count','batch'].includes(k)).map(k=>[k,avg(tcpRaw.map(row=>row[k]))]))});
  const fixture = fixtures.find(profile=>profile.codec===codec);
  assert.equal(fixture.records.length, 256);
  const received = [];
  for (const run of http) {
    assert.equal(run.exitCode, 0);
    const raw = await read('results/realistic/http/' + run.id + '.json');
    assert.equal(raw.codec, codec);
    assert.equal(raw.vus, 8);
    assert.equal(raw.measurement_seconds, 8);
    const metrics = raw.data.metrics;
    assert.equal(metrics.verified.values.rate, 1);
    assert.equal(metrics.http_req_failed.values.rate, 0);
    const requests = metrics.measured_requests.values.count;
    assert(requests > 0);
    const time = metrics.measured_latency_ms.values;
    received.push({pass:run.pass, requests, rps:requests / raw.measurement_seconds,
      p50_ms:time.med, p95_ms:time['p(95)'], p99_ms:time['p(99)'],
      binary_body_bytes:metrics.measured_binary_body_bytes.values.count/requests});
  }
  httpRows.push({codec, trials:received, requests:received.reduce((sum,row)=>sum+row.requests,0),
    rps:avg(received.map(row=>row.rps)), rps_range:received.map(row=>row.rps),
    p50_ms:avg(received.map(row=>row.p50_ms)), p95_ms:avg(received.map(row=>row.p95_ms)), p99_ms:avg(received.map(row=>row.p99_ms)),
    drift_percent:drift(received.map(row=>row.rps)),
    api_body_bytes:avg(fixture.records.map(r=>r.request+r.response)),
    request_bytes:avg(fixture.records.map(r=>r.request)), response_bytes:avg(fixture.records.map(r=>r.response)),
  });
}

const data = {names, workloads, corpora, codecs, codecRows, stringRows, native, nativeMeta:featureProbe.native,
  httpRows, tcpRows, realistic, features, transport, measuredHttpRequests:httpRows.reduce((sum,row)=>sum+row.requests,0)};
const numeric = x => Number(x).toFixed(2);
const us = ns => numeric(ns / 1000);
const separator = n => '|' + Array(n).fill('---').join('|') + '|\n';
const table = (headers, rows) => '|'+headers.join('|')+'|\n'+separator(headers.length) + rows.map(row=>'|'+row.join('|')+'|').join('\n')+'\n\n';
const nativeLabels = mode => names[mode] ?? mode;
function csv(name, rows) {
  const flat = rows.map(row=>Object.fromEntries(Object.entries(row).filter(([,value])=>value==null||typeof value!=='object')));
  return save(name, [Object.keys(flat[0]).join(','), ...flat.map(row=>Object.values(row).map(value=>JSON.stringify(value??'')).join(','))].join('\n'));
}
await Promise.all([
  save('results/realistic/scenario-summary.json', JSON.stringify(data, null, 2)),
  csv('results/realistic/codec.csv', codecRows),
  csv('results/realistic/http.csv', httpRows.flatMap(row=>row.trials.map(trial=>({codec:row.codec, ...trial})))),
  csv('results/realistic/socket.csv', tcpRows.flatMap(row=>row.trials.map(trial=>({codec:row.codec, ...trial})))),
  csv('results/qsl-features/strings.csv', stringRows),
  csv('results/qsl-features/native.csv', native.map(({id,codec,median_ns,runs_ns,ci99_ns,drift_percent})=>({
    id, codec, median_ns, pass1_ns:runs_ns[0], pass2_ns:runs_ns[1],
    pass1_ci_low_ns:ci99_ns[0][0], pass1_ci_high_ns:ci99_ns[0][1],
    pass2_ci_low_ns:ci99_ns[1][0], pass2_ci_high_ns:ci99_ns[1][1], drift_percent,
  }))),
]);

let doc = '# Прикладные сценарии QSL 0.4.0\n\n';
doc += 'Проверяемая схема имитирует API телеметрии. В ней один из трёх payload API (Reading, Update, Notice), а для сокетов предусмотрены Heartbeat и Event. Вложенные Context, Geo, Auth, Patch, Limits и Retry имеют независимое присутствие. Набор из 256 записей на сценарий включает пустые строки, нули, false, пустые списки и отсутствие необязательных полей. Все варианты сверены по полной структуре. [Описание модели](realistic/src/model.rs) · [HTML с фильтрами и расчётом](dashboard.html)\n\n';
doc += 'QSL кодирует тип payload явным дискриминантом. Вложенные переменные записи кодируются сгенерированными проверяемыми QSL-кодеками и вкладываются как массивы байтов с длиной. Время и аллокации этой сборки учтены. Proto3 использует oneof и explicit presence. Cap’n Proto использует union, признаки присутствия для скаляров и указатели для вложенных записей. Bincode использует Rust enum и Option, конфигурация standard / varint. Cap’n Proto измерен в обычном и packed режиме.\n\n';
doc += '## Только кодек\n\nCriterion на прогретом кольце. Запись принимает готовую структуру и выделяет выходной кадр. Клонирование входа в setup вынесено из таймера. Чтение создаёт общую владеющую Rust-структуру. Приведены средние двух медиан. Замеры аллокаций выполнены отдельным бинарником, байты означают сумму запрошенных размеров alloc/realloc.\n\n';
for (const workload of Object.keys(workloads)) {
  doc += '### '+workloads[workload]+'\n\n';
  doc += table(['Кодек','Кадр, B','Запись, мкс','Чтение, мкс','alloc запись','alloc чтение','Дрейф, %'],codecRows.filter(r=>r.workload===workload)
    .map(r=>[names[r.codec],numeric(r.wire_mean),us(r.encode_ns),us(r.decode_ns),numeric(r.encode_allocs),numeric(r.decode_allocs),numeric(r.drift_percent)]));
}
doc += '## HTTP API\n\nPOST /v1/normalize принимает бинарный запрос, создаёт модель, проверяет ветку, увеличивает revision и возвращает новый бинарный кадр. k6 предварительно загружает те же 256 запросов, затем проверяет каждый байт ответа. HTTP/1.1 с keep-alive на localhost, 8 VU, 2 с прогрева и по 8 с измерения, два противоположных порядка. Числа включают HTTP-путь и работу клиента. Модель без TLS и базы данных. Короткие задержки под Windows иногда попадают ниже разрешения таймера k6.\n\n';
doc += table(['Кодек','Запросов/с','Диапазон двух прогонов','p95, мс','Запрос + ответ, B'],httpRows.map(r=>[names[r.codec],numeric(r.rps),r.rps_range.map(numeric).join(' / '),numeric(r.p95_ms),numeric(r.api_body_bytes)]));
doc += 'В измерительной фазе проверены '+data.measuredHttpRequests+' запросов. HTTP-ошибок и расхождений содержимого не обнаружено.\n\n';
doc += '## TCP Heartbeat / Events\n\nОдин постоянный TCP-сокет на испытание. Клиент отправляет Heartbeat и проверяет ответ с новой ревизией, затем одним вызовом записи отправляет пакет из 32 Events и ждёт ACK. Сервер декодирует все события и проверяет ветку, возвращая число и контрольную сумму идентификаторов и списка affected. Форматы делят одинаковую длину-префикс 4 B. ACK занимает 12 B + 4 B префикса и учитывается в среднем объёме на событие. RTT исключает подготовку отправляемых кадров и клиентскую сверку. Events/с вычислены по активному времени пакетов с ACK. 128 прогревочных и 2048 измеренных циклов в двух порядках.\n\n';
doc += table(['Кодек','Heartbeat RTT p95, мкс','Heartbeat в обе стороны, B','32 Events + ACK p95, мкс','Events/с','Полный B на Event'],tcpRows.map(r=>[names[r.codec],numeric(r.heartbeat_p95_us),numeric(r.heartbeat_bytes),numeric(r.batch_p95_us),numeric(r.active_events_per_s),numeric(r.event_bytes)]));
doc += '## Расчёт трафика\n\nКалькулятор в HTML суммирует HTTP-тела запроса и ответа, Heartbeat/pong и Events с префиксами и ACK внутри TCP. Для каждого сценария средний размер умножается на выбранную частоту. Результат в GiB за сутки включает полезные данные обоих направлений. HTTP-заголовки, TCP/IP, TLS, ретрансляции и сжатие транспорта находятся вне расчёта.\n\n';
const daily = codec => {
  const http = httpRows.find(row=>row.codec===codec);
  const tcp = tcpRows.find(row=>row.codec===codec);
  return [http.api_body_bytes*1000, tcp.heartbeat_bytes*10000/30, tcp.event_bytes*10000*.5].map(rate=>rate*86400/2**30);
};
const protoDaily = daily('protobuf').reduce((sum,part)=>sum+part,0);
doc += 'Пример модели нагрузки: 1 000 запросов API в секунду и 10 000 соединений, по одному Heartbeat каждые 30 с и 0,5 Event/с на соединение. Это расчёт по размерам кадров, а не достигнутая ёмкость localhost. Отрицательная разница означает уменьшение объёма против Protobuf.\n\n';
doc += table(['Кодек','HTTP, GiB/сутки','Heartbeat, GiB/сутки','Events, GiB/сутки','Всего, GiB/сутки','Разница к Protobuf, GiB'],
  codecs.map(codec=>{
    const parts = daily(codec);
    const total = parts.reduce((sum,part)=>sum+part,0);
    return [names[codec],...parts.map(numeric),numeric(total),numeric(total-protoDaily)];
  }));
doc += '[Тайминги и аллокации](results/realistic/codec.csv) · [HTTP](results/realistic/http.csv) · [TCP](results/realistic/socket.csv) · [Сырые выборки](results/realistic)\n';
await save('REALISTIC.md', doc.trimEnd());

let featureDoc = '# QSL 0.4.0 — трансмутация, StackString и QSL-LZ\n\n';
featureDoc += '## Трансмутация\n\nФиксированная структура 16 B с выравниванием 8 B. Генератор подтверждает совпадение Rust-представления и кадра на little-endian машине. Все пути проверяют диапазон вложенного поля и читают одинаковые значения. Копирование возвращает самостоятельный объект, overlay заимствует вход. Для overlay проверено отклонение смещённого на 1 байт адреса. [Схемы](qsl-features/schemas/features.qsl)\n\n';
featureDoc += table(['Путь чтения','Время, нс','Дрейф, %'],native.map(row=>[nativeLabels(row.codec),numeric(row.median_ns),numeric(row.drift_percent)]));
featureDoc += '## Строки\n\n256 записей по 1024 байта на корпус. Запись использует готовый объект и буфер. Чтение создаёт владеющий объект и вычисляет XXH3 по всему UTF-8. Размер объекта включает встроенное хранилище, но у String дополнительные 1024 байта выделяются отдельно. StackString сохраняет тот же wire. QSL-LZ работает через InlineString. При неудачном коротком поиске совпадения используется raw с двухбайтным заголовком. Повторяющиеся строки журнала с длинным периодом и переменным префиксом демонстрируют это поведение, а период 16 байт проверяет сжатую ветку.\n\n';
for (const corpus of Object.keys(corpora)) {
  featureDoc += '### '+corpora[corpus]+'\n\n';
  featureDoc += table(['Строка','Кадр, B','Объект, B','Запись, мкс','Чтение, мкс','alloc чтение','Дрейф, %'],stringRows.filter(r=>r.workload===corpus)
    .map(r=>[names[r.codec],numeric(r.wire_mean),r.object_bytes,us(r.encode_ns),us(r.decode_ns),numeric(r.decode_allocs),numeric(r.drift_percent)]));
}
featureDoc += 'Все замеры Criterion имеют повтор в обратном порядке. CSV по трансмутации включает CI 99% двух проходов, сырые данные по строкам сохраняют те же оценки и выборки. [HTML](dashboard.html) · [Строки](results/qsl-features/strings.csv) · [Трансмутация](results/qsl-features/native.csv) · [Сырые выборки](results/qsl-features)\n';
await save('QSL-FEATURES.md',featureDoc.trimEnd());
const template = await fs.readFile(path.join(root, 'tools/dashboard-template.html'),'utf8');
const htmlData = {...data, realistic:{runs:realistic.runs, environment:{cpu:realistic.environment.cpu}}, features:{runs:features.runs}};
await save('dashboard.html', template.replace('/* BENCHMARK_DATA */', () => JSON.stringify(htmlData).replaceAll('<', '\\u003c')).trimEnd());
console.log('Verified reports', {codecOperations:realistic.rows.length, featureOperations:features.rows.length,
  httpRequests:data.measuredHttpRequests, socketTrials:socketSamples.length});
