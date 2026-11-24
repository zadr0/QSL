import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {gzipSync, gunzipSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const batches = [1, 8, 64, 256];
const families = ['QSL', 'Protobuf', 'Cap’n Proto', 'Bincode'];
const workloadNames = {
  api: ['Polymorphic API', 'Полиморфный API'],
  heartbeat: ['Heartbeat', 'Heartbeat'],
  event: ['Socket event', 'Событие сокета'],
  fixed: ['Four bounded integers', 'Четыре ограниченных числа'],
  text_128: ['128-byte text', 'Текст 128 Б'],
  text_2048: ['2,048-byte text', 'Текст 2048 Б'],
};
const operations = {
  allocate: ['Allocate and encode', 'Аллокация и запись'],
  buffer: ['Encode into a ready buffer', 'Запись в готовый буфер'],
  owned: ['Decode to an owned value', 'Чтение с владением'],
  view: ['Zero-copy text view', 'Чтение zero-copy'],
};
const file = relative => path.join(root, relative);
const save = (relative, data) => fs.writeFile(file(relative), JSON.stringify(data, null, 2) + '\n');
const read = async relative => {
  const bytes = await fs.readFile(file(relative));
  return JSON.parse(bytes.toString(bytes[0] === 255 && bytes[1] === 254 ? 'utf16le' : 'utf8')
    .replace(/^\uFEFF/, ''));
};
const sha256 = async name => createHash('sha256').update(await fs.readFile(name)).digest('hex').toUpperCase();
const average = values => values.reduce((sum, value) => sum + value, 0) / values.length;
const family = profile => families.find(name => profile.startsWith({
  QSL: 'qsl_', Protobuf: 'protobuf', 'Cap’n Proto': 'capnp', Bincode: 'bincode'
}[name]));
const details = id => {
  const [workload, operation, profile, size] = id.split('/');
  assert(workloadNames[workload] && operations[operation] && family(profile), id);
  return {id, workload, operation, profile, family: family(profile), batch: size ? Number(size) : null};
};
const group = row => [row.workload, row.operation, row.batch].join('/');

async function criterion(directory, captureSamples = false) {
  const result = [];
  async function walk(relative) {
    const files = await fs.readdir(file(relative), {withFileTypes: true});
    if (files.some(entry => entry.name === 'benchmark.json')) {
      const benchmark = await read(relative + '/benchmark.json');
      const estimates = await read(relative + '/estimates.json');
      const sample = captureSamples ? await read(relative + '/sample.json') : null;
      result.push({benchmark, estimates, sample});
      return;
    }
    for (const entry of files) {
      if (entry.isDirectory() && !['base', 'change', 'report'].includes(entry.name)) {
        await walk(relative + '/' + entry.name);
      }
    }
  }
  await walk(directory);
  const ordered = result.sort((a, b) => a.benchmark.full_id.localeCompare(b.benchmark.full_id));
  assert.equal(new Set(ordered.map(row => row.benchmark.full_id)).size, ordered.length);
  return ordered;
}

async function pickBatch() {
  const directory = 'results/optimized-batch';
  const candidates = (await criterion(directory + '/tune/criterion')).map(raw => ({
    ...details(raw.benchmark.full_id), tune_ns: raw.estimates.median.point_estimate,
  }));
  assert.equal(candidates.length, 199 * batches.length);
  const winners = new Map();
  for (const row of candidates) {
    assert(batches.includes(row.batch) && row.tune_ns > 0, row.id);
    const key = group(row) + '/' + row.family;
    const current = winners.get(key);
    if (!current || row.tune_ns < current.tune_ns) winners.set(key, row);
  }
  const picked = [...winners.values()].sort((a, b) => a.id.localeCompare(b.id));
  assert.equal(picked.length, 56 * batches.length);
  await save(directory + '/selection.json', picked.map(row => row.id));
  await save(directory + '/allocation-selection.json', [...new Set(picked.map(row => row.id.slice(0, row.id.lastIndexOf('/'))))]);
  await save(directory + '/tuning.json', {candidates, picked});
  console.log('Locked', picked.length, 'batch profiles for confirmation');
}

async function collect(directory, candidates, finalists, probe = null) {
  let raw;
  try {
    raw = Object.fromEntries(await Promise.all(['tune', 'confirm-1', 'confirm-2'].map(async phase =>
      [phase, await criterion(directory + '/' + phase + '/criterion', true)])));
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
    raw = JSON.parse(gunzipSync(await fs.readFile(file(directory + '/criterion.json.gz'))));
  }
  const runs = await read(directory + '/runs.json');
  const manifest = await read(directory + '/selection.json');
  const tuning = await read(directory + '/tuning.json');
  const ids = rows => rows.map(row => row.benchmark.full_id).sort();
  assert.equal(runs.length, 3);
  assert(runs.every(run => run.exitCode === 0));
  assert.deepEqual(runs.map(run => run.name), ['tune', 'confirm-1', 'confirm-2']);
  assert.equal(raw.tune.length, candidates);
  assert.deepEqual(ids(raw.tune), tuning.candidates.map(row => row.id).sort());
  assert.equal(manifest.length, finalists);
  assert.deepEqual(manifest.toSorted(), tuning.picked.map(row => row.id).sort());
  for (const phase of ['confirm-1', 'confirm-2']) {
    assert.deepEqual(ids(raw[phase]), manifest.toSorted());
  }
  const run1 = new Map(raw['confirm-1'].map(row => [row.benchmark.full_id, row.estimates.median]));
  const run2 = new Map(raw['confirm-2'].map(row => [row.benchmark.full_id, row.estimates.median]));
  const baseId = row => row.batch ? row.id.slice(0, row.id.lastIndexOf('/')) : row.id;
  const probeById = new Map(probe.profiles.map(row => [row.id, row.samples]));
  assert.equal(probeById.size, probe.profiles.length);
  assert.deepEqual([...probeById.keys()].sort(), [...new Set(manifest.map(id => baseId(details(id))))].sort());
  const rows = manifest.map(id => {
    const [first, second] = [run1.get(id), run2.get(id)].map(median => {
      const {lower_bound: low, upper_bound: high, confidence_level: level} = median.confidence_interval;
      assert.equal(level, 0.99);
      assert(low > 0 && median.point_estimate >= low && median.point_estimate <= high, id);
      return {ns: median.point_estimate, low, high};
    });
    const row = {...details(id), first, second, ns: average([first.ns, second.ns])};
    row.nsPerRecord = row.ns / (row.batch ?? 1);
    const samples = probeById.get(baseId(row));
    assert.equal(samples.length, 256);
    row.wireBytes = average(samples.map(sample => sample.wire_bytes));
    row.allocationCalls = average(samples.map(sample => sample.alloc_calls));
    row.allocationBytes = average(samples.map(sample => sample.alloc_bytes));
    if (row.batch) row.allocationBytesPerBatch = row.allocationBytes * row.batch;
    return row;
  });
  const groups = new Map();
  for (const row of rows) {
    if (!groups.has(group(row))) groups.set(group(row), []);
    groups.get(group(row)).push(row);
  }
  for (const results of groups.values()) {
    assert.deepEqual(results.map(row => row.family).sort(), families.toSorted());
    for (const row of results) {
      const faster = (a, b) => a.first.high < b.first.low && a.second.high < b.second.low;
      const others = results.filter(other => other !== row);
      row.outcome = others.every(other => faster(row, other)) ? 'lead'
        : others.some(other => faster(other, row)) ? 'behind' : 'unresolved';
    }
  }
  await fs.writeFile(file(directory + '/criterion.json.gz'), gzipSync(JSON.stringify(raw)));
  return {rows, runs};
}

function csv(rows) {
  const columns = [
    'workload', 'operation', 'batch', 'family', 'profile', 'ns', 'nsPerRecord',
    'pass1_ns', 'pass1_ci99_low', 'pass1_ci99_high', 'pass2_ns', 'pass2_ci99_low',
    'pass2_ci99_high', 'wireBytes', 'allocationCalls', 'allocationBytes', 'allocationBytesPerBatch', 'outcome',
  ];
  const cell = val => typeof val === 'number' ? String(+val.toFixed(5)) : (val ?? '');
  return [columns.join(','), ...rows.map(row =>
    columns.map(key => cell(({
      pass1_ns: row.first.ns, pass1_ci99_low: row.first.low, pass1_ci99_high: row.first.high,
      pass2_ns: row.second.ns, pass2_ci99_low: row.second.low, pass2_ci99_high: row.second.high,
    })[key] ?? row[key])).join(','))].join('\n') + '\n';
}

const round = value => value.toLocaleString('en-US', {maximumFractionDigits: 2});
async function render(summary, batchesSummary) {
  const template = await fs.readFile(file('tools/optimized-dashboard-template.html'), 'utf8');
  for (const [index, lang] of ['en', 'ru'].entries()) {
    const payload = JSON.stringify({summary, batchesSummary, workloadNames, operations, index})
      .replaceAll('<', '\\u003c');
    await fs.writeFile(file(lang === 'en' ? 'dashboard.en.html' : 'dashboard.html'),
      template.replace('@@LANG@@', lang).replace('@@PAYLOAD@@', () => payload));
  }
  const qslRows = summary.rows.filter(row => row.family === 'QSL');
  const tally = rows => ['lead', 'behind', 'unresolved'].map(outcome =>
    rows.filter(row => row.outcome === outcome).length);
  const describe = ([lead, behind, inconclusive]) =>
    `**${lead} confirmed leads, ${behind} confirmed deficits and ${inconclusive} inconclusive comparisons** for QSL.`;
  const sections = Object.keys(operations).map(operation => {
    const header = `### ${operations[operation][0]}\n\n`;
    const rows = qslRows.filter(row => row.operation === operation);
    const table = ['| Workload | QSL profile | QSL | Protobuf | Cap’n Proto | Bincode | QSL result |',
      '|---|---|---:|---:|---:|---:|---|'];
    for (const row of rows) {
      const values = families.map(f => round(summary.rows.find(item => group(item) === group(row) && item.family === f).ns));
      table.push(`| ${workloadNames[row.workload][0]} | \`${row.profile}\` | ${values.join(' | ')} | ${row.outcome} |`);
    }
    return header + table.join('\n');
  });
  const batchRows = batchesSummary?.rows.filter(row => row.family === 'QSL') ?? [];
  const batchReport = batchRows.length ? [
    '## Batch results',
    '',
    describe(tally(batchRows)),
    '',
    '| Messages per batch | QSL leads | QSL behind | Inconclusive |',
    '|---:|---:|---:|---:|',
    ...batches.map(size => `| ${size} | ${tally(batchRows.filter(row => row.batch === size)).join(' | ')} |`),
    '',
    'The dashboard shows latency per batch and per message. Each iteration processes all its independent frames. Buffer writers reuse one 32 KiB destination and consume the written prefix before proceeding to the next frame. The timer includes the batch loop and cursor update. Batch frames keep the same individual wire format. Requested bytes per batch are the mean per-message allocation requests multiplied by the batch size.',
    '',
    '[Batch measurements](results/optimized-batch/confirmed.csv) · [Batch calibration](results/optimized-batch/tuning.json) · [Batch Criterion samples](results/optimized-batch/criterion.json.gz)',
    '',
  ].join('\n') : '';
  const docs = await fs.readFile(file('tools/optimized-methodology.md'), 'utf8');
  const environment = summary.environment;
  const windows = environment.os.Caption.match(/Windows.*/i)?.[0] ?? 'Windows';
  const machine = `${environment.cpu.Name.trim()}, ${windows} ${environment.os.Version}, ${environment.rust.split('\n')[0].trim()}`;
  const text = docs.replace('<!-- RESULTS -->', () => [
    describe(tally(qslRows)), '', 'All timings below are in **ns per record**. Each cell is the mean of the two independent Criterion medians.', '',
    ...sections.flatMap(section => [section, '']), batchReport,
  ].join('\n')).replace('<!-- MACHINE -->', () => machine)
    .replace('<!-- LOAD -->', () => [['single record',summary.runs],['batches',batchesSummary?.runs]]
      .filter(([,runs]) => runs)
      .map(([label,runs]) => label + ' (' + runs.map(run =>
        `${run.name} ${round(run.backgroundLogicalCores)}`).join(', ') + ')').join(' and '));
  await fs.writeFile(file('OPTIMIZED.md'), text);
}

async function main() {
  if (process.argv[2] === 'pick-batch') return pickBatch();
  const directory = 'results/optimized';
  const environment = await read(directory + '/environment.json');
  for (const [name, digest] of Object.entries(environment.sources)) {
    assert.equal(await sha256(file(name)), digest, name);
  }
  assert.equal(await sha256(file(directory + '/generated.qsl')), environment.generatedSchemaSha256);
  if (path.isAbsolute(environment.executable)) {
    assert.equal(await sha256(environment.executable), environment.executableSha256);
    environment.executable = path.relative(root, environment.executable).replaceAll('\\', '/');
    assert(environment.executable.startsWith('target/'));
  }
  const probe = await read(directory + '/profiles.json');
  const summary = {...await collect(directory, 199, 56, probe), environment, bmi2: probe.bmi2};
  await save(directory + '/summary.json', summary);
  await fs.writeFile(file(directory + '/confirmed.csv'), csv(summary.rows));

  let batchesSummary;
  if (process.argv[2] !== 'single') {
    const batchDirectory = 'results/optimized-batch';
    const batchEnvironment = await read(batchDirectory + '/environment.json');
    for (const [name, digest] of Object.entries(batchEnvironment.sources)) {
      assert.equal(await sha256(file(name)), digest, name);
    }
    const batchProbe = await read(batchDirectory + '/profiles.json');
    batchesSummary = {...await collect(batchDirectory, 796, 224, batchProbe), environment: batchEnvironment};
    await save(batchDirectory + '/summary.json', batchesSummary);
    await fs.writeFile(file(batchDirectory + '/confirmed.csv'), csv(batchesSummary.rows));
  }
  await render(summary, batchesSummary);
  console.log('Rendered', summary.rows.length, 'single-record and', batchesSummary?.rows.length ?? 0, 'batch rows');
}

await main();
