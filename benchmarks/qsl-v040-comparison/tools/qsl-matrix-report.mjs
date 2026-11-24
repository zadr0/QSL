import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {gzipSync, gunzipSync} from 'node:zlib';
import {fileURLToPath} from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const result = 'results/qsl-matrix';
const phases = ['tune', 'confirm-1', 'confirm-2'];
const target = name => path.join(root,name);
const output = name => target(result+'/'+name);
const json = async name => JSON.parse((await fs.readFile(output(name),'utf8')).replace(/^\uFEFF/,''));
const save = (name, value) => fs.writeFile(output(name), JSON.stringify(value,null,2)+'\n');
const ns = n => n.toLocaleString('en-US',{maximumFractionDigits:2});

async function criterion(phase) {
  const parent = output(phase+'/criterion');
  const entries = await fs.readdir(parent,{withFileTypes:true});
  const cases = await Promise.all(entries.filter(e=>e.isDirectory()).map(async entry=>{
    const directory = path.join(parent,entry.name,'new');
    const read = async name=>JSON.parse(await fs.readFile(path.join(directory,name+'.json'),'utf8'));
    const [definition,estimates,sample] = await Promise.all([read('benchmark'),read('estimates'),read('sample')]);
    const {point_estimate,confidence_interval} = estimates.median;
    assert(point_estimate>0,definition.full_id);
    assert.equal(confidence_interval.confidence_level,0.99,definition.full_id);
    return {id:definition.full_id,ns:point_estimate,ci:[confidence_interval.lower_bound,confidence_interval.upper_bound],estimates,sample};
  }));
  assert.equal(new Set(cases.map(row=>row.id)).size,cases.length);
  return cases.sort((a,b)=>a.id.localeCompare(b.id));
}

const query = id => {
  const [workload,profile,operation] = id.split('/');
  assert(operation,id);
  return {workload,profile,operation};
};
const isSafety = ({workload,profile,operation}) => {
  if (profile === 'borrowed_aligned') return ['text_1024','text_8192'].includes(workload);
  if (profile === 'owned') return ['text_32','text_1024','text_8192','utf8_8192'].includes(workload);
  if (profile === 'borrowed') return true;
  if (workload === 'dyn') return true;
  if (workload === 'domain') return profile === 'auto' || profile === 'compact';
  return ['fixed/auto','fixed/compact','fixed/compact_group_native','presence/auto','presence/ps_ot',
    'presence/bitmap','periodic_1024/qsl_lz_inline','noise_1024/qsl_lz_inline'].includes(workload+'/'+profile)
    && !operation.endsWith('_lease');
};

async function select() {
  const tune = await criterion('tune');
  const profiles = await json('tune/profiles.json');
  const sizes = await json('tune/sizes.json');
  assert(tune.length>400);
  for (const measured of profiles.filter(row=>row.status==='measured')) {
    assert(sizes.some(row=>row.group===measured.group&&row.profile===measured.profile),measured.profile);
  }
  const layoutOf = new Map(sizes.map(row=>[row.workload+'/'+row.profile,row.group]));
  const winners = new Map();
  const anchors = new Set(['auto','declared','fields','heap_string','line_16','qps_q2']);
  const calibration = tune.map(row=>({...query(row.id),id:row.id,ns:row.ns}));
  for (const row of calibration) {
    const {workload,profile,operation} = row;
    const group = layoutOf.get(workload+'/'+profile);
    // The three Safety string outputs and overlay borrow separate kinds of result.
    if (group==='safety_text' || profile==='transmute_overlay' || !/^(checked|trusted)_(read|write)$/.test(operation)) continue;
    const key = workload+'/'+operation;
    if (!winners.has(key) || row.ns < winners.get(key).ns) winners.set(key,row);
  }
  const picked = [...winners.values()];
  const bench = calibration.filter(row=>isSafety(row) ||
    (anchors.has(row.profile) && /^(checked|trusted)_(read|write)$/.test(row.operation)) ||
    ['cached_write','rt_lease_write','transmute_copy_read'].includes(row.operation) ||
    row.profile==='transmute_overlay');
  const selection = [...new Set([...picked,...bench].map(row=>row.id))].sort();
  assert(selection.length<tune.length);
  await save('selection.json',selection);
  await save('calibration.json',{picked,candidates:calibration});
  console.log('Selected',selection.length,'of',tune.length,'for independent confirmation');
}

function csv(rows, keys) {
  const cell = val => typeof val==='number'?String(+val.toFixed(7)):String(val??'');
  return keys.join(',')+'\n'+rows.map(row=>keys.map(key=>cell(row[key])).join(',')).join('\n')+'\n';
}

const normal = ['checked_write','trusted_write','checked_read','trusted_read'];
const costs = ['checked_read','trusted_read','validate','validate_then_trusted','checked_write','trusted_write','validate_values'];
const captions = {checked_write:'Checked write',trusted_write:'Trusted write',checked_read:'Checked read',trusted_read:'Trusted read',
  validate:'Validate only',validate_then_trusted:'Validate + trusted read',validate_values:'Validate values',
  cached_write:'CachedFrame write + copy',rt_lease_write:'QSL-RT lease write + copy',transmute_copy_read:'Native copy',
};
function grid(headers, values) {
  return ['| '+headers.join(' | ')+' |','| '+headers.map(()=> '---').join(' | ')+' |',
    ...values.map(row=>'| '+row.join(' | ')+' |')].join('\n');
}

async function render(bundle) {
  const {environment,runs,profiles,sizes,timed,tuning,winners,safety,extras} = bundle;
  const rejected = profiles.filter(row=>row.status==='rejected');
  const quantized = profiles.filter(row=>row.quant_active);
  const pext = profiles.filter(row=>row.emits_pext);
  const featureTable = grid(['Workload','Operation','Selected profile','ns / record','Frame B','Object B'],
    winners.map(r=>[r.workload,captions[r.operation],'`'+r.profile+'`',ns(r.ns),ns(r.mean_wire_bytes),r.object_bytes]));
  const extraTable = grid(['Workload','Profile','Operation','ns / record','Frame B','Object B'],
    extras.map(r=>[r.workload,'`'+r.profile+'`',captions[r.operation]??r.operation,ns(r.ns),ns(r.mean_wire_bytes),r.object_bytes]));
  const featureNote = (await fs.readFile(target('tools/qsl-all-features.md'),'utf8'))
    .replace('<!-- TOTAL -->',()=>`${profiles.length} schema configurations, ${profiles.filter(row=>row.status==='measured').length} measured profiles, ${tuning.length} calibration cases and ${timed.length} independently measured finalists`)
    .replace('<!-- TABLE -->',()=>featureTable).replace('<!-- EXTRA -->',()=>extraTable)
    .replace('<!-- TRADEOFFS -->',()=>{
      const get=(workload,profile)=>tuning.find(row=>row.workload===workload&&row.profile===profile);
      const array=get('arrays','inline'), arrayShift=get('arrays','inline_shifted');
      const heap=get('periodic_1024','heap_string'), zip=get('periodic_1024','qsl_lz_inline');
      const stack=get('periodic_1024','stack_string'), stackShift=get('periodic_1024','shifted_stack_string');
      if(!array||!arrayShift||!heap||!zip||!stack||!stackShift) return '';
      return `On this target, shifted lengths changed the inline-array object from **${array.object_bytes} B** to **${arrayShift.object_bytes} B**, and the inline-string object from **${stack.object_bytes} B** to **${stackShift.object_bytes} B**. Their respective wire sizes stayed the same. For periodic strings, average frame size went from **${ns(heap.mean_wire_bytes)} B** with heap String to **${ns(zip.mean_wire_bytes)} B** with inline QSL-LZ. Those averages include the 25% of fixtures with the text absent.`;
    })
    .replace('<!-- PLANS -->',()=>[
      `QPS was active in **${quantized.length}** profiles. Code for a BMI2 PEXT path was found in **${pext.length}** generated profiles. The profile manifest retains these plan facts for each candidate.`,
      '',
      ...rejected.map(r=>`- \`${r.profile}\` · ${r.reason.replace(/\s+/g,' ')}`),
    ].join('\n'))
    .replace('<!-- ENV -->',()=>`${environment.cpu.trim()}, Windows ${environment.os}, ${environment.rust}. Mean background load in logical cores was ${runs.map(r=>r.phase+' '+ns(r.backgroundLogicalCores)).join(', ')}.`);
  const safetyTable = grid(['Workload','Profile',...costs.map(k=>captions[k]),'Read ratio'],
    safety.map(row=>[row.workload,'`'+row.profile+'`',...costs.map(op=>row[op] ? ns(row[op]) : '—'),ns(row.readRatio)+'×']));
  const safetyNote = (await fs.readFile(target('tools/qsl-safety.md'),'utf8'))
    .replace('<!-- SAFETY -->',()=>safetyTable)
    .replace('<!-- EXAMPLE -->',()=>{
      const pair=(workload,profile)=>safety.find(row=>row.workload===workload&&row.profile===profile);
      const small=pair('text_32','borrowed'), large=pair('text_8192','borrowed');
      const utf=pair('utf8_8192','borrowed'), owned=pair('text_8192','owned');
      if(!small||!large||!utf||!owned) return '';
      return `At 32 ASCII bytes, checked borrowed read took **${ns(small.checked_read)} ns** and trusted borrowed read took **${ns(small.trusted_read)} ns**. At 8,192 ASCII bytes the corresponding times were **${ns(large.checked_read)} ns** and **${ns(large.trusted_read)} ns**. Validation alone took **${ns(large.validate)} ns** at this length. The 8,192-byte multibyte corpus took **${ns(utf.validate)} ns** to validate. Owning the 8,192-byte ASCII string raised trusted time to **${ns(owned.trusted_read)} ns** for the owned profile.`;
    });
  await fs.writeFile(target('QSL-ALL-FEATURES.md'),featureNote);
  await fs.writeFile(target('QSL-SAFETY.md'),safetyNote);
  const template = await fs.readFile(target('tools/qsl-matrix-dashboard.html'),'utf8');
  await fs.writeFile(target('qsl-matrix.html'),template.replace('@@DATA@@',()=>JSON.stringify({winners,safety,profiles,tuning,extras,captions}).replaceAll('<','\\u003c')));
}

async function report(){
  const environment = await json('environment.json');
  for(const [name,expected] of Object.entries(environment.sources)){
    assert.equal(createHash('sha256').update(await fs.readFile(target(name))).digest('hex').toUpperCase(),expected,name);
  }
  const runs = await json('runs.json');
  assert.deepEqual(runs.map(row=>row.phase),phases);
  assert(runs.every(row=>row.exitCode===0));
  const calibration=await json('calibration.json');
  const candidates=calibration.candidates;
  const selection=await json('selection.json');
  let raw;
  try {
    raw=Object.fromEntries(await Promise.all(phases.map(async phase=>[phase,await criterion(phase)])));
  } catch(error) {
    if(error.code!=='ENOENT') throw error;
    raw=JSON.parse(gunzipSync(await fs.readFile(output('criterion.json.gz'))));
  }
  assert.deepEqual(raw.tune.map(row=>row.id).sort(),candidates.map(row=>row.id).sort());
  for(const phase of phases.slice(1)) assert.deepEqual(raw[phase].map(row=>row.id).sort(),selection.toSorted());
  let profiles,sizes,calibrationSizes;
  try {
    profiles = await json('tune/profiles.json');
    calibrationSizes = await json('tune/sizes.json');
    sizes = await json('confirm-1/sizes.json');
    await fs.copyFile(output('tune/schemas.qsl'),output('schemas.qsl'));
    await save('profiles.json',profiles);
    await save('calibration-sizes.json',calibrationSizes);
    await save('sizes.json',sizes);
  }catch(error){
    if(error.code!=='ENOENT') throw error;
    profiles=await json('profiles.json');
    calibrationSizes=await json('calibration-sizes.json');
    sizes=await json('sizes.json');
  }
  const second=new Map(raw['confirm-2'].map(row=>[row.id,row]));
  const memory=new Map(sizes.map(row=>[row.workload+'/'+row.profile,row]));
  const timed=raw['confirm-1'].map(first=>{
    const other=second.get(first.id);
    const {workload,profile,operation}=query(first.id);
    const cost=memory.get(workload+'/'+profile);
    assert(cost,first.id);
    return {...cost,operation,id:first.id,ns:(first.ns+other.ns)/2,pass1_ns:first.ns,pass1_low:first.ci[0],pass1_high:first.ci[1],
      pass2_ns:other.ns,pass2_low:other.ci[0],pass2_high:other.ci[1]};
  });
  const times = new Map(timed.map(row=>[row.id,row]));
  const winners = calibration.picked.map(row=>times.get(row.id));
  const extras = timed.filter(row=>!normal.includes(row.operation) && !costs.includes(row.operation) || row.profile==='transmute_overlay');
  const safety=[...new Set(timed.filter(row=>isSafety(row)).map(row=>row.workload+'/'+row.profile))].map(id=>{
    const [workload,profile]=id.split('/');
    const row={workload,profile};
    for(const operation of costs) row[operation]=times.get(id+'/'+operation)?.ns;
    assert(row.checked_read && row.trusted_read,id);
    row.readRatio=row.checked_read/row.trusted_read;
    return row;
  });
  const calibrationMemory=new Map(calibrationSizes.map(row=>[row.workload+'/'+row.profile,row]));
  const tuning=candidates.map(row=>{
    const footprint=calibrationMemory.get(row.workload+'/'+row.profile);
    assert(footprint,row.id);
    return {...footprint,...row};
  });
  const bundle={environment,runs,profiles,sizes,timed,tuning,winners,safety,extras};
  await fs.writeFile(output('criterion.json.gz'),gzipSync(JSON.stringify(raw)));
  await fs.writeFile(output('confirmed.csv'),csv(timed,['group','workload','profile','operation','ns','mean_wire_bytes','object_bytes','align',
    'pass1_ns','pass1_low','pass1_high','pass2_ns','pass2_low','pass2_high']));
  await fs.writeFile(output('calibration.csv'),csv(tuning,
    ['group','workload','profile','operation','ns','mean_wire_bytes','object_bytes','align']));
  await fs.writeFile(output('safety.csv'),csv(safety,['workload','profile',...costs,'readRatio']));
  await save('summary.json',bundle);
  await render(bundle);
  console.log('Rendered',timed.length,'timings,',winners.length,'feature winners and',safety.length,'safety pairs');
}

await (process.argv[2]==='select' ? select() : report());
