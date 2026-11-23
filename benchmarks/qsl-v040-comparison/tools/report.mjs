import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import * as vega from 'vega';
import * as vl from 'vega-lite';

const root=fileURLToPath(new URL('../',import.meta.url));
const result=path.join(root,'results','released');
const read=async p=>{const raw=await fs.readFile(p);const text=raw[0]===255&&raw[1]===254?raw.toString('utf16le'):raw.toString('utf8');return JSON.parse(text.replace(/^\uFEFF/,''));};
const environment=await read(path.join(result,'environment.json'));
let runs=await read(path.join(result,'runs.json'));if(!Array.isArray(runs))runs=[runs];
if(runs.length!==2||runs.some(r=>r.exitCode!==0))throw Error('Exactly two successful runs are required');
const profiles=await read(path.join(result,'profiles.json'));
const datasets=[];
for(let index=1;index<=2;index++){
 const folder=path.join(result,runs[index-1].run,'criterion'),map=new Map();
 for(const entry of await fs.readdir(folder,{withFileTypes:true})){
  if(!entry.isDirectory())continue;
  const current=path.join(folder,entry.name,'new');
  let benchmark,estimates;
  try{benchmark=await read(path.join(current,'benchmark.json'));estimates=await read(path.join(current,'estimates.json'));}catch{continue;}
  if(map.has(benchmark.full_id))throw Error('Duplicate measurement');
  map.set(benchmark.full_id,estimates);
 }
 if(map.size!==100)throw Error('Expected 100 measurements in run '+index+', received '+map.size);
 datasets.push(map);
}
const measurements=[];
for(const [id,a]of datasets[0]){
 const b=datasets[1].get(id);if(!b)throw Error('Missing reverse run: '+id);
 const parts=id.split('/'),workload=parts[0],operation=parts[1],codec=parts[2]??'consumer_only';
 const p=profiles.profiles.find(p=>p.workload===workload&&p.codec===codec&&(operation==='write_prepared'?p.write_measured:p.read_mode===operation));
 if(operation!=='consume_control'&&!p)throw Error('Missing metadata for '+id);
 const median1=a.median.point_estimate,median2=b.median.point_estimate;
 const mean=(median1+median2)/2;
 measurements.push({id,workload,operation,codec,median1_ns:median1,median2_ns:median2,mean_of_medians_ns:mean,min_median_ns:Math.min(median1,median2),max_median_ns:Math.max(median1,median2),
  run1_ci99_low:a.median.confidence_interval.lower_bound,run1_ci99_high:a.median.confidence_interval.upper_bound,
  run2_ci99_low:b.median.confidence_interval.lower_bound,run2_ci99_high:b.median.confidence_interval.upper_bound,
  drift_percent:100*Math.abs(median1-median2)/mean,
  wire_mean:p?.wire_mean??null,wire_min:p?.wire_min??null,wire_max:p?.wire_max??null,
  allocations:p?(operation==='write_prepared'?p.write_allocations:p.read_allocations):null,
  allocated_bytes:p?(operation==='write_prepared'?p.write_allocated_bytes:p.read_allocated_bytes):null,
 });
}
const integrity=[];
for(const [relative,hash]of Object.entries(environment.sources)){
 const actual=createHash('sha256').update(await fs.readFile(path.join(root,relative.replaceAll('\\','/')))).digest('hex').toUpperCase();
 if(actual!==hash)integrity.push(relative);
}
const binaryHash=createHash('sha256').update(await fs.readFile(environment.executable)).digest('hex').toUpperCase();
if(binaryHash!==environment.executableSha256)integrity.push('benchmark executable');
if(integrity.length)throw Error('Measured sources changed: '+integrity.join(', '));
const family=codec=>codec.startsWith('qsl')?'QSL':codec.startsWith('protobuf')?'Protobuf':codec.startsWith('bincode')?'Bincode':codec.startsWith('capnp')?'Cap’n Proto':'Control';
for(const row of measurements)row.family=family(row.codec);
const summary={generatedAt:new Date().toISOString(),qslVersion:profiles.qsl_version,environment,runs,profiles,measurements,sourceIntegrity:'unchanged',aggregation:'arithmetic mean of two independent per-run medians; not a pooled confidence interval',
 driftOver10Percent:measurements.filter(r=>r.drift_percent>10).map(r=>({id:r.id,drift_percent:r.drift_percent}))};
await fs.writeFile(path.join(result,'summary.json'),JSON.stringify(summary,null,2));
const columns=Object.keys(measurements[0]);
const csvCell=value=>value===null?'':typeof value==='string'?'"'+value.replaceAll('"','""')+'"':String(value);
await fs.writeFile(path.join(result,'measurements.csv'),columns.join(',')+'\n'+measurements.map(r=>columns.map(k=>csvCell(r[k])).join(',')).join('\n')+'\n');
const labels={write_prepared:'Запись в готовый буфер',read_owned:'Полное чтение с владением',read_borrowed:'Полное чтение с заимствованием'};
const groups=['numeric','optionals','text_32','text_256','text_1024'];
const nf=value=>value==null?'—':value.toFixed(2);
const tables=[];
for(const workload of groups){
 const rows=measurements.filter(r=>r.workload===workload&&r.operation!=='consume_control');
 const codecs=[...new Set(rows.map(r=>r.codec))];
 const data=codecs.map(codec=>{
  const related=rows.filter(r=>r.codec===codec);
  const get=op=>related.find(r=>r.operation===op);
  return{codec,wire:related[0].wire_mean,wireMin:related[0].wire_min,wireMax:related[0].wire_max,write:get('write_prepared'),owned:get('read_owned'),borrowed:get('read_borrowed')};
 });
 tables.push({workload,rows:data});
}
const figureDir=path.join(result,'figures');await fs.mkdir(figureDir,{recursive:true});
const figures=[];
for(const workload of groups){
 for(const operation of Object.keys(labels)){
  const rows=measurements.filter(r=>r.workload===workload&&r.operation===operation);
  if(!rows.length)continue;
  const spec={
   $schema:'https://vega.github.io/schema/vega-lite/v6.json',
   background:'#0b0e14',width:470,height:{step:27},padding:18,
   title:{text:workload+' · '+labels[operation],subtitle:'нс/запись · меньше — быстрее; отрезок: медианы двух прогонов',anchor:'start',fontSize:17,subtitleFontSize:11,color:'#f8fafc',subtitleColor:'#94a3b8',offset:18},
   data:{values:rows},
   encoding:{y:{field:'codec',type:'nominal',sort:rows.toSorted((a,b)=>a.mean_of_medians_ns-b.mean_of_medians_ns).map(r=>r.codec),title:null,axis:{labelLimit:250,labelFontSize:11}},color:{field:'family',scale:{domain:['QSL','Protobuf','Bincode','Cap’n Proto'],range:['#6aa6ff','#3edab0','#f6c56f','#ed72b8']},legend:null}},
   layer:[
    {mark:{type:'bar',cornerRadiusEnd:3,opacity:0.75},encoding:{x:{field:'mean_of_medians_ns',type:'quantitative',title:'нс на запись',scale:{zero:true,domainMax:Math.max(...rows.map(r=>r.max_median_ns))*1.23}}}},
    {mark:{type:'rule',color:'#f8fafc',strokeWidth:2},encoding:{x:{field:'min_median_ns',type:'quantitative'},x2:{field:'max_median_ns'}}},
    {mark:{type:'text',align:'left',baseline:'middle',dx:7,color:'#e2e8f0',fontSize:11},encoding:{x:{field:'max_median_ns',type:'quantitative'},text:{field:'mean_of_medians_ns',type:'quantitative',format:'.2f'},color:{value:'#e2e8f0'}}}
   ],
   config:{font:'Arial',view:{stroke:null},axis:{labelColor:'#cbd5e1',titleColor:'#cbd5e1',gridColor:'#243041',domain:false,tickColor:'#334155',titleFontWeight:'normal'}}
  };
  const runtime=vega.parse(vl.compile(spec).spec);
  const view=new vega.View(runtime,{renderer:'none'}).initialize();
  let svg=await view.toSVG();view.finalize();
  for(const match of [...svg.matchAll(/\bid="([^"]+)"/g)]){const old=match[1],next=workload+"-"+operation+"-"+old;svg=svg.replaceAll('id="'+old+'"','id="'+next+'"').replaceAll("url(#"+old+")","url(#"+next+")").replaceAll('href="#'+old+'"','href="#'+next+'"');}
  const filename=workload+'-'+operation+'.svg';await fs.writeFile(path.join(figureDir,filename),svg);
  figures.push({workload,operation,filename,svg});
 }
}
const title='# QSL v0.4.0: сравнение с Protobuf, Cap’n Proto и Bincode\n\n';
let markdown=title+'Измерено 23 ноября 2026 на '+environment.cpu.Name.trim()+', Windows '+environment.os.Version+', Rust 1.98.0. 24 GiB RAM, 2133 MHz. Все библиотеки собраны одинаково: native CPU, thin LTO, один codegen unit.\n\n';
markdown+='Два противоположно упорядоченных прогона, 100 случаев в каждом, по 200 выборок; прогрев 1 с, измерение 3 с. Числа ниже — среднее двух медиан, нс/запись. Это не объединённый доверительный интервал. Медианы и CI 99% каждого прохода сохранены в [CSV](results/released/measurements.csv).\n\n';
markdown+='Запуск каждого прохода разрешён после трёх последовательных наблюдений общей загрузки CPU ≤20%; процесс закреплён на ядре 2. Проверка простоя не гарантирует отсутствие фоновых всплесков во время всего прохода. Средняя нагрузка вне процесса составила '+runs.map(r=>r.otherSystemCpuCoreEquivalent==null?'недоступно':r.otherSystemCpuCoreEquivalent.toFixed(3)+' ядра').join(' / ')+'. Сравнения небольших различий требуют осторожности; строк с дрейфом между медианами >10%: '+summary.driftOver10Percent.length+'.\n\n';
markdown+='Версии: QSL 0.4.0, qsl-core 0.2.0, Prost 0.14.4, capnp 0.27.3, Bincode 2.0.1, Criterion 0.8.2. Все 5 проверок контрактов прошли в debug и native release; 100 случаев прошли smoke-проверку; хэши исходников и исполняемого файла не изменились.\n\n';
const pick=(workload,operation,codec)=>measurements.find(r=>r.workload===workload&&r.operation===operation&&r.codec===codec);
markdown+='## Наблюдения\n\n';
markdown+='- Четыре u32: QSL default даёт '+nf(pick('numeric','write_prepared','qsl_default').mean_of_medians_ns)+' нс записи и '+nf(pick('numeric','read_owned','qsl_default').mean_of_medians_ns)+' нс полного чтения при кадре 16 B. compact уменьшает кадр до 6 B, но чтение здесь занимает '+nf(pick('numeric','read_owned','qsl_compact').mean_of_medians_ns)+' нс. Размер и скорость чтения меняются по-разному.\n';
markdown+='- scalar compact и group/BMI2 записывают тот же шестибайтный кадр за '+nf(pick('numeric','write_prepared','qsl_compact').mean_of_medians_ns)+' / '+nf(pick('numeric','write_prepared','qsl_compact_group_bmi2').mean_of_medians_ns)+' нс. Наличие PEXT подтверждено; близкие числа не доказывают выигрыша BMI2 на этом CPU.\n';
markdown+='- Optional-чтение QSL default / PS+OT: '+nf(pick('optionals','read_owned','qsl_default').mean_of_medians_ns)+' / '+nf(pick('optionals','read_owned','qsl_ps_ot').mean_of_medians_ns)+' нс. Bincode fixint в этой структуре читает за '+nf(pick('optionals','read_owned','bincode_fixint').mean_of_medians_ns)+' нс при большем кадре 10.5 B.\n';
markdown+='- Owned-чтение UTF-8 строки 1024 B у четырёх базовых кодеков близко к 1 мкс. Разница в несколько процентов зависит от шума и условий. QSL inline убирает heap-аллокацию, сохраняя данные внутри объекта 1032 B.\n';
markdown+='- Cap’n Proto write в этом контракте включает построение нового сообщения из подготовленных значений, а затем запись стандартного frame. Запись уже построенного сообщения имела бы другой контракт и здесь не измерена. Все Cap’n Proto кадры unpacked.\n\n';
markdown+='Раздельные отчёты: [одинаковые структуры](SAME-STRUCTURES.md), [только QSL](QSL-ONLY.md), [все Dyn](DYN.md), [k6](K6.md).\n\n';
markdown+='## Полная матрица\n\nКаждая строка восстанавливает те же значения. Размер — среднее по 256 входам; для переменных размеров диапазон указан в скобках. owned и borrowed — разные контракты. Аллокации чтения — среднее число обращений к системному аллокатору на запись, отдельно от тайминга.\n\n';
for(const {workload,rows}of tables){
 markdown+='### '+workload+'\n\n| Вариант | Frame, B | Запись, нс | Чтение owned, нс | Чтение borrowed, нс | Аллокации чтения owned / borrowed |\n|---|---:|---:|---:|---:|---:|\n';
 for(const row of rows){const wire=nf(row.wire)+(row.wireMin===row.wireMax?'':' ('+row.wireMin+'–'+row.wireMax+')');markdown+='| '+row.codec+' | '+wire+' | '+nf(row.write?.mean_of_medians_ns)+' | '+nf(row.owned?.mean_of_medians_ns)+' | '+nf(row.borrowed?.mean_of_medians_ns)+' | '+nf(row.owned?.allocations)+' / '+nf(row.borrowed?.allocations)+' |\n';}
 markdown+='\n';
}
markdown+='## Стоимость общего потребителя\n\nЭти числа не вычитаются из остальных результатов.\n\n| Данные | Consumer-only, нс |\n|---|---:|\n';
for(const row of measurements.filter(r=>r.operation==='consume_control'))markdown+='| '+row.workload+' | '+nf(row.mean_of_medians_ns)+' |\n';
markdown+='\n## Как читать результаты\n\n- Compact у QSL использует объявленный 12-битный домен: 16 B → 6 B. Alignment округляет каждое 12-битное поле до 16 бит: итог 8 B. В release-дизассемблировании подтверждены PEXT для варианта group/BMI2.\n- PS измерен на схеме с различимыми суммами длин optional-полей 1, 2, 4 B. Его выигрыш и применимость зависят от этой структуры; OT отдельно не обязан ускорять каждый reader.\n- Cap’n Proto u16 и Bincode u16 явно используют более узкое физическое представление того же допустимого домена. Учитывается полный стандартный frame Cap’n Proto.\n- Для строк String/StackString результаты владеют данными; lifetime, Bincode borrow и Cap’n Proto view заимствуют их. У Cap’n Proto owned текст копируется в String, у borrowed этого копирования нет.\n- QSL inline-объект занимает '+profiles.qsl_inline_object_bytes+' B, heap-объект '+profiles.qsl_heap_object_bytes+' B плюс данные в heap, borrowed-объект '+profiles.qsl_borrow_object_bytes+' B. Отсутствие heap-аллокации не означает меньший native-объект.\n- Cap’n Proto write использует scratch-память, но message builder сохраняет собственную служебную аллокацию. Она входит во время и отдельно отражена в profiles.json.\n- Измерены прогретые кодеки и ограниченное кольцо данных, а не сеть или холодная память. Для иной машины, размеров и распределений возможен другой порядок.\n\n';
markdown+='Подробный протокол и команды: [README](README.md). [HTML с графиками](report.html), [JSON](results/released/summary.json), [метаданные/аллокации](results/released/profiles.json), [условия](results/released/environment.json), [прогоны](results/released/runs.json), [PEXT](results/released/bmi2-instructions.txt).\n';
await fs.writeFile(path.join(root,'RESULTS.md'),markdown);
const esc=value=>String(value??'').replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;');
let sections='';
for(const {workload,rows}of tables){
 sections+='<section data-workload="'+workload+'"><h2>'+workload+'</h2><div class="charts">'+figures.filter(f=>f.workload===workload).map(f=>'<figure>'+f.svg+'</figure>').join('')+'</div><div class="table-wrap"><table><thead><tr><th>Вариант</th><th>Frame, B</th><th>Запись, нс</th><th>Owned, нс</th><th>Borrowed, нс</th><th>Аллокации owned / borrowed</th></tr></thead><tbody>';
 for(const row of rows){sections+='<tr><td>'+esc(row.codec)+'</td><td>'+nf(row.wire)+(row.wireMin===row.wireMax?'':' <small>('+row.wireMin+'–'+row.wireMax+')</small>')+'</td><td>'+nf(row.write?.mean_of_medians_ns)+'</td><td>'+nf(row.owned?.mean_of_medians_ns)+'</td><td>'+nf(row.borrowed?.mean_of_medians_ns)+'</td><td>'+nf(row.owned?.allocations)+' / '+nf(row.borrowed?.allocations)+'</td></tr>';}
 sections+='</tbody></table></div></section>';
}
const html='<!doctype html><html lang="ru"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>QSL 0.4.0 · Serializer comparison</title><style>:root{color-scheme:dark}*{box-sizing:border-box}body{margin:0;background:#0b0e14;color:#dce4f0;font:15px/1.6 system-ui,sans-serif}main{max-width:1560px;margin:auto;padding:40px 28px}h1{font-size:clamp(25px,4vw,45px);line-height:1.15;margin:0 0 14px}h2{font-size:24px;margin-top:44px}p{max-width:1000px;color:#aab9cc}a{color:#6aa6ff}.legend{display:flex;gap:24px;flex-wrap:wrap;margin:22px 0}.legend span:before{content:"";display:inline-block;width:9px;height:9px;border-radius:2px;background:var(--color);margin-right:7px}select{padding:10px 14px;background:#151c28;color:#e2e8f0;border:1px solid #35445a;border-radius:7px;font:inherit}.charts{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,610px),1fr));gap:16px}figure{margin:0;border:1px solid #243041;border-radius:12px;padding:4px;overflow:hidden}svg{width:100%;height:auto;display:block}.table-wrap{overflow:auto;margin-top:24px;border:1px solid #243041;border-radius:10px}table{width:100%;border-collapse:collapse;font-variant-numeric:tabular-nums;font-size:13px}td,th{padding:12px 15px;border-bottom:1px solid #243041;text-align:right;white-space:nowrap}td:first-child,th:first-child{text-align:left}th{background:#141b26;color:#f8fafc}small{color:#7f91a7}footer{margin-top:40px;padding-top:18px;border-top:1px solid #243041;color:#94a3b8}[hidden]{display:none!important}</style><main><h1>QSL 0.4.0 · сравнение кодеков</h1><p>Protobuf / Prost 0.14.4 · Cap’n Proto 0.27.3 · Bincode 2.0.1<br>Intel Core i5-7400 · Windows 11 · Rust 1.98.0 · 23 ноября 2026</p><p>Среднее двух независимых медиан, нс на запись; меньше — быстрее. Отрезки показывают различие медиан двух проходов. Owned и borrowed имеют разные контракты. 100 случаев × 2 прогона; по 200 выборок; прогрев 1 с, измерение 3 с, CI 99% в исходных данных. Перед каждым проходом: три наблюдения общего CPU ≤20%; закрепление на ядре 2; одно закреплённое ядро.</p><div class="legend"><span style="--color:#6aa6ff">QSL</span><span style="--color:#3edab0">Protobuf</span><span style="--color:#f6c56f">Bincode</span><span style="--color:#ed72b8">Cap’n Proto</span></div><label>Набор данных <select id="workload"><option value="all">Все наборы</option>'+groups.map(g=>'<option>'+g+'</option>').join('')+'</select></label>'+sections+'<footer>Данные и ограничения: <a href="README.md">методика</a> · <a href="RESULTS.md">полный текст</a> · <a href="results/released/measurements.csv">CSV с медианами и CI</a> · <a href="results/released/summary.json">JSON</a><p>Числа и UTF-8 проверяются, все поля потребляются. Затраты общего consumer не вычитаются. Измерения относятся к прогретому набору на этой машине; это не универсальный рейтинг библиотек.</p></footer></main><script>document.getElementById("workload").addEventListener("change",e=>{document.querySelectorAll("section[data-workload]").forEach(s=>s.hidden=e.target.value!=="all"&&s.dataset.workload!==e.target.value)});</script></html>';
await fs.writeFile(path.join(root,'report.html'),html);
console.log(JSON.stringify({measurements:measurements.length,profiles:profiles.profiles.length,figures:figures.length,sourceIntegrity:'unchanged',driftOver10Percent:summary.driftOver10Percent.length},null,2));
