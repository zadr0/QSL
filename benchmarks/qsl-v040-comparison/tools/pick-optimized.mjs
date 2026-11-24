import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const output = path.join(root, 'results/optimized');
const candidates = [];
const family = profile => {
  if (profile.startsWith('qsl_')) return 'QSL';
  if (profile.startsWith('protobuf')) return 'Protobuf';
  if (profile.startsWith('bincode')) return 'Bincode';
  if (profile.startsWith('capnp')) return 'Cap’n Proto';
  throw Error('Unknown encoding profile: ' + profile);
};
const folder = path.join(output, 'tune/criterion');
for (const entry of await fs.readdir(folder, {withFileTypes:true})) {
  if (!entry.isDirectory()) continue;
  const raw = path.join(folder, entry.name, 'new');
  const benchmark = JSON.parse(await fs.readFile(path.join(raw,'benchmark.json'),'utf8'));
  const statistics = JSON.parse(await fs.readFile(path.join(raw,'estimates.json'),'utf8'));
  const id = benchmark.full_id;
  const [workload, operation, profile] = id.split('/');
  assert(profile && Number.isFinite(statistics.median.point_estimate));
  candidates.push({id,workload,operation,profile,family:family(profile),tune_ns:statistics.median.point_estimate});
}
assert.equal(new Set(candidates.map(row=>row.id)).size,199);
assert.equal(candidates.length,199);
const groups = new Map();
for (const row of candidates) {
  const key=[row.workload,row.operation,row.family].join('/');
  const current=groups.get(key);
  if(!current || row.tune_ns<current.tune_ns || (row.tune_ns===current.tune_ns && row.id<current.id))groups.set(key,row);
}
const picked=[...groups.values()].sort((a,b)=>a.id.localeCompare(b.id));
assert.equal(picked.length,56);
await fs.writeFile(path.join(output,'selection.json'),JSON.stringify(picked.map(row=>row.id),null,2)+'\n');
await fs.writeFile(path.join(output,'tuning.json'),JSON.stringify({candidates,picked},null,2)+'\n');
console.log('Locked',picked.length,'profiles for an independent confirmation corpus');
