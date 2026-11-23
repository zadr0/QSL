import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import * as vega from 'vega';
import * as vl from 'vega-lite';
const root=fileURLToPath(new URL('../',import.meta.url));
const result=path.join(root,'results/k6');
const read=async p=>JSON.parse((await fs.readFile(p,'utf8')).replace(/^\uFEFF/,''));
const environment=await read(path.join(result,'environment.json'));
const runs=await read(path.join(result,'runs.json'));
if(runs.length!==144||runs.some(r=>r.exitCode!==0))throw Error('Expected 136 closed + 8 open successful k6 trials');
for(const [relative,hash]of Object.entries(environment.sources)){
 const actual=createHash('sha256').update(await fs.readFile(path.join(root,relative.replaceAll('\\','/')))).digest('hex').toUpperCase();
 if(actual!==hash)throw Error('Changed k6 measured source '+relative);
}
const manifest=await read(path.join(result,'fixtures/manifest.json'));
if(manifest.length!==33)throw Error('Expected 33 HTTP codec profiles');
for(const workload of ['numeric','optionals','text_1024']){
 const profiles=manifest.filter(p=>p.id.startsWith(workload+'/'));
 const reference=profiles[0].records;
 for(const profile of profiles){
  if(profile.records.length!==256||profile.records.some((r,i)=>r.digest!==reference[i].digest))throw Error('Logical input mismatch: '+profile.id);
 }
}
const rows=[];
for(const run of runs){
 const raw=await read(path.join(result,'trials',run.name+'.json'));
 const metrics=raw.data.metrics;
 if(!metrics?.measured_requests?.values?.count)throw Error('Missing request data '+run.name);
 if(metrics.verified?.values?.rate!==1||metrics.http_req_failed?.values?.rate!==0||metrics.measured_verification_failures?.values?.count!==0)throw Error('Verification failure '+run.name);
 const t=metrics.measured_latency_ms.values;
 const [workload,codec]=raw.profile.split('/');
 const frames=manifest.find(p=>p.id===raw.profile).records;
 rows.push({name:run.name,pass:run.pass,model:raw.model,workload,codec,operation:raw.operation,
  measured_seconds:environment.measurementSeconds,requests:metrics.measured_requests.values.count,
  requests_per_second:metrics.measured_requests.values.count/environment.measurementSeconds,
  latency_median_ms:t.med,latency_average_ms:t.avg,latency_p95_ms:t['p(95)'],latency_p99_ms:t['p(99)'],
  http_failure_rate:metrics.http_req_failed.values.rate,verification_failure_count:metrics.measured_verification_failures.values.count,
  dropped_iterations:metrics.dropped_iterations?.values?.count??0,
  wire_mean:frames.reduce((n,f)=>n+f.length,0)/frames.length,
  server_cpu_seconds:run.serverCpuSeconds,client_cpu_seconds:run.clientCpuSeconds,
  other_cpu_core_equivalent:run.otherCpuCoreEquivalent,elapsed_seconds:run.elapsedSeconds});
}
const groupMap=new Map();
for(const row of rows.filter(r=>r.model==='closed')){
 const key=[row.workload,row.codec,row.operation].join('/');
 if(!groupMap.has(key))groupMap.set(key,[]);
 groupMap.get(key).push(row);
}
const combined=[];
for(const [id,values]of groupMap){
 if(values.length!==2)throw Error('Two closed runs missing '+id);
 const [a,b]=values;
 const avg=k=>(a[k]+b[k])/2,min=k=>Math.min(a[k],b[k]),max=k=>Math.max(a[k],b[k]);
 combined.push({id,workload:a.workload,codec:a.codec,operation:a.operation,wire_mean:a.wire_mean,
  rps_mean:avg('requests_per_second'),rps_min:min('requests_per_second'),rps_max:max('requests_per_second'),
  p95_mean_ms:avg('latency_p95_ms'),p95_min_ms:min('latency_p95_ms'),p95_max_ms:max('latency_p95_ms'),
  p99_mean_ms:avg('latency_p99_ms'),requests:a.requests+b.requests,
  rps_drift_percent:100*(max('requests_per_second')-min('requests_per_second'))/avg('requests_per_second')});
}
if(combined.length!==68)throw Error('Expected 68 HTTP endpoint/operation combinations');
const cols=Object.keys(rows[0]);
await fs.writeFile(path.join(result,'measurements.csv'),cols.join(',')+'\n'+rows.map(r=>cols.map(c=>typeof r[c]==='string'?JSON.stringify(r[c]):r[c]).join(',')).join('\n')+'\n');
await fs.writeFile(path.join(result,'summary.json'),JSON.stringify({environment,runs:rows,combined,aggregation:'arithmetic means of separate run RPS/p95; no pooled quantile or CI'},null,2));
const nf=x=>Number(x).toFixed(2);
const family=codec=>codec.startsWith('dyn_')||codec.startsWith('qsl_')?'QSL':
 codec.startsWith('protobuf')?'Protobuf':codec.startsWith('bincode')?'Bincode':'Cap’n Proto';
const openRuns=rows.filter(r=>r.model==='open');
const totalRequests=rows.reduce((n,r)=>n+r.requests,0);
let doc='# k6: HTTP-нагрузка на одинаковых структурах и всех Dyn-режимах\n\n';
doc+='**144 завершённых испытания без ошибок содержимого**: 33 профиля × запись/чтение × два противоположных порядка, два транспортных контроля × два порядка, восемь дополнительных испытаний с заданной частотой. k6 2.2.0, локальный Rust HTTP/1.1-сервер, keep-alive, без TLS. Ни один запрос не направлялся на production-сайт.\n\n';
doc+='На испытание: 2 с прогрева, 1 с разделения фаз, '+environment.measurementSeconds+' с измерения. Основной closed model: '+environment.closedVUs+' VU, без think time. Сервер закреплён на логическом ядре 2, k6 — на 0/1/3. Оба процесса находятся на одном i5-7400; это измеряет весь демонстрационный HTTP-путь и работу генератора нагрузки, а не только кодек, не удалённую сеть и не Cap’n Proto RPC/gRPC.\n\n';
doc+='**Фоновая нагрузка:** вне сервера и k6 она занимала от '+nf(Math.min(...rows.map(r=>r.other_cpu_core_equivalent)))+' до '+nf(Math.max(...rows.map(r=>r.other_cpu_core_equivalent)))+' логических ядер. Эти испытания не заявляются как замеры на полностью свободной машине. CPU клиента, сервера и остальных процессов записан для каждого испытания; ограничителем может быть генератор нагрузки.\n\n';
doc+='## Контракт\n\n`encode`: GET с индексом записи; сервер сериализует заранее подготовленный native-объект в повторно используемый буфер. k6 сравнивает **каждый** ответный байт с заранее проверенным кадром.\n\n`decode`: POST настоящего бинарного кадра; сервер выполняет checked-декодирование и общий checksum всех полей/всех байтов текста, создаёт owned-результат и уничтожает его. k6 сравнивает 64-битный результат с эталоном. В каждом профиле циклически проходят все 256 записей. Временная память Dyn размещается на стеке и входит в чтение.\n\nТранспортные контроли возвращают готовый кадр/готовый checksum без кодека; их время не вычитается. Данные, HTTP-парсер, потоки и обработчик одинаковы для протоколов. На клиенте проверка encode зависит от длины кадра; её стоимость влияет на достигнутый RPS.\n\n';
doc+='RPS = число завершённых запросов измерительной фазы / заданное окно. Средние RPS и p95 ниже усредняют два отдельных испытания, а **не** объединяют выборки или квантили. Диапазоны показывают два результата. Исходные p50/p95/p99, ошибки, CPU и счётчики находятся в [CSV](results/k6/measurements.csv).\n\n';
if(rows.some(r=>r.latency_median_ms===0))doc+='**Разрешение таймера:** на этой машине k6 зарегистрировал 0 мс в '+rows.filter(r=>r.latency_median_ms===0).length+' медианах и '+rows.filter(r=>r.latency_p95_ms===0).length+' значениях p95. Задержки ниже разрешения измерения следует читать как «ниже разрешения». Эти значения сохранены в CSV. Время самого кодека измерено отдельно в Criterion.\n\n';
function table(data){
 let s='| Кодек / режим | Кадр, B | RPS: среднее (диапазон) | p95, мс: среднее (диапазон) | Дрейф RPS, % |\n|---|---:|---:|---:|---:|\n';
 for(const r of data)s+='| '+r.codec+' | '+nf(r.wire_mean)+' | '+nf(r.rps_mean)+' ('+nf(r.rps_min)+'–'+nf(r.rps_max)+') | '+nf(r.p95_mean_ms)+' ('+nf(r.p95_min_ms)+'–'+nf(r.p95_max_ms)+') | '+nf(r.rps_drift_percent)+' |\n';
 return s+'\n';
}
for(const workload of ['numeric','optionals','text_1024']){
 doc+='## '+workload+'\n\n';
 for(const operation of ['encode','decode']){
  doc+='![HTTP '+workload+' '+operation+'](results/k6/figures/'+workload+'-'+operation+'.svg)\n\n';
  doc+='### '+operation+': одинаковые структуры\n\n'+table(combined.filter(r=>r.workload===workload&&r.operation===operation&&!r.codec.startsWith('dyn_')));
  doc+='### '+operation+': все режимы Dyn\n\n'+table(combined.filter(r=>r.workload===workload&&r.operation===operation&&r.codec.startsWith('dyn_')));
 }
}
doc+='## Контроли транспорта без кодека\n\n'+table(combined.filter(r=>r.operation.startsWith('control_')).map(r=>({...r,codec:r.operation})));
doc+='## Open model: 1000 запросов/с\n\nОтдельные constant-arrival-rate испытания для чтения numeric и text_1024; 32 заранее выделенных VU, 10 с измерения. dropped_iterations означает, что запланированная итерация не была начата, а не ошибку декодирования. Эти испытания проверяют достижение заданной частоты; они не устанавливают максимальную пропускную способность.\n\n';
doc+='| Структура | Кодек | Достигнутые запросы/с | p95, мс | Dropped |\n|---|---|---:|---:|---:|\n';
for(const r of openRuns)doc+='| '+r.workload+' | '+r.codec+' | '+nf(r.requests_per_second)+' | '+nf(r.latency_p95_ms)+' | '+r.dropped_iterations+' |\n';
doc+='\nБез пропусков прошли '+openRuns.filter(r=>r.dropped_iterations===0).length+' из '+openRuns.length+' испытаний. Всего пропущено '+openRuns.reduce((sum,r)=>sum+r.dropped_iterations,0)+' запланированных итераций.\n';
doc+='\nВсего проверенных запросов измерительных фаз: '+totalRequests+'. HTTP-ошибок и ошибок содержимого: 0. Разброс между прогонами и близость к транспортным контролям ограничивают выводы о небольших различиях кодеков. Для изолированного времени используйте [одинаковые структуры](SAME-STRUCTURES.md), [QSL](QSL-ONLY.md) и [Dyn](DYN.md).\n\n';
doc+='[JSON](results/k6/summary.json) · [полные k6 summary](results/k6/trials) · [сервер](http/src/main.rs) · [скрипт нагрузки](k6/load.js) · [документация k6](https://grafana.com/docs/k6/latest/using-k6/scenarios/executors/constant-arrival-rate/).\n';
await fs.writeFile(path.join(root,'K6.md'),doc);
const figures=path.join(result,'figures');await fs.mkdir(figures,{recursive:true});
for(const workload of ['numeric','optionals','text_1024'])for(const operation of ['encode','decode']){
 const data=combined.filter(r=>r.workload===workload&&r.operation===operation).map(r=>({...r,family:family(r.codec)}));
 const spec={$schema:'https://vega.github.io/schema/vega-lite/v6.json',width:600,height:{step:25},background:'#0b0e14',padding:20,
  title:{text:workload+' · HTTP '+operation+' · '+environment.closedVUs+' VU',subtitle:'запросов/с · отрезок показывает результаты двух испытаний',anchor:'start',color:'#f8fafc',subtitleColor:'#94a3b8'},
  data:{values:data},encoding:{y:{field:'codec',type:'nominal',sort:data.toSorted((a,b)=>b.rps_mean-a.rps_mean).map(r=>r.codec),title:null},x:{field:'rps_mean',type:'quantitative',title:'запросов/с'}},
  layer:[{mark:{type:'bar',opacity:0.85,cornerRadiusEnd:3},encoding:{color:{field:'family',type:'nominal',scale:{domain:['QSL','Protobuf','Bincode','Cap’n Proto'],range:['#6aa6ff','#3edab0','#f6c56f','#ed72b8']},legend:null}}},{mark:{type:'rule',color:'#fff',strokeWidth:2},encoding:{x:{field:'rps_min',type:'quantitative'},x2:{field:'rps_max'}}}],
  config:{font:'Arial',view:{stroke:null},axis:{labelColor:'#cbd5e1',titleColor:'#cbd5e1',gridColor:'#243041',labelLimit:240}}};
 const view=new vega.View(vega.parse(vl.compile(spec).spec),{renderer:'none'}).initialize();
 const svg=await view.toSVG();view.finalize();
 await fs.writeFile(path.join(figures,workload+'-'+operation+'.svg'),svg);
}
console.log('k6 report verified:',rows.length,'trials,',totalRequests,'measured requests');
