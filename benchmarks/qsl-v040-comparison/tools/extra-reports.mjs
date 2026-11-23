import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const root=fileURLToPath(new URL('../',import.meta.url));
const read=async p=>JSON.parse((await fs.readFile(path.join(root,p),'utf8')).replace(/^\uFEFF/,''));
const nf=x=>x==null?'—':Number(x).toFixed(2);
const csv=rows=>{
 const cols=Object.keys(rows[0]);
 const cell=x=>x==null?'':typeof x==='string'?JSON.stringify(x):String(x);
 return cols.join(',')+'\n'+rows.map(r=>cols.map(k=>cell(r[k])).join(',')).join('\n')+'\n';
};
const base=await read('results/released/summary.json');
const n=base.measurements;
const groups=['numeric','optionals','text_32','text_256','text_1024'];
const table=rows=>{
 const keys=[...new Set(rows.map(r=>r.codec))];
 let s='| Схема / кодек | Кадр, B | Запись, нс | Owned, нс | Borrowed, нс | Макс. дрейф, % |\n|---|---:|---:|---:|---:|---:|\n';
 for(const key of keys){
  const p=rows.filter(r=>r.codec===key),get=op=>p.find(r=>r.operation===op)?.mean_of_medians_ns;
  s+='| '+key+' | '+nf(p[0].wire_mean)+' | '+nf(get('write_prepared'))+' | '+nf(get('read_owned'))+' | '+nf(get('read_borrowed'))+' | '+nf(Math.max(...p.map(r=>r.drift_percent)))+' |\n';
 }
 return s+'\n';
};
const conditions='Intel Core i5-7400, Windows 11, Rust 1.98.0, target-cpu=native, thin LTO, codegen-units=1. Два противоположных порядка, 1 с прогрева + 3 с измерения, 200 выборок на случай, CI 99%. Среднее двух независимых медиан; исходные CI не объединяются. Фоновая нагрузка записана, система не считается полностью простаивающей.\n\n';
const baseline=n.filter(r=>{
 const key=r.workload==='numeric'?['qsl_default','protobuf_varint','bincode_fixint','capnp_u32']:
 r.workload==='optionals'?['qsl_default','protobuf','bincode_fixint','capnp']:
 ['qsl_heap','protobuf','bincode_fixint','capnp'];
 return key.includes(r.codec)&&['write_prepared','read_owned'].includes(r.operation);
});
let same='# Одинаковые структуры: QSL / Protobuf / Bincode / Cap’n Proto\n\n'+conditions;
same+='Выбран фиксированный baseline для каждого протокола. Узкие u16, borrowed-результаты, QSL compact/PS/OT/BM и альтернативные кодировки исключены из этой таблицы. Полная матрица вариантов доступна в [общем отчёте](RESULTS.md). Здесь используются измеренные строки того же полного прохода, без повторного округления или другого набора данных.\n\n';
same+='| Структура | Общий контракт | Отображение на протоколы |\n|---|---|---|\n| numeric | четыре u32, значения 0..4095 | QSL u32; Protobuf uint32; Bincode u32 fixint; Cap’n Proto UInt32 |\n| optionals | id:u32, a:Option<u8>, b:Option<u16>, c:Option<u32> | Protobuf использует optional uint32 с проверкой диапазонов; Cap’n Proto — явные флаги присутствия и UInt8/16/32. Some(0) сохраняется |\n| text | id:u32, String UTF-8, до 1024 байт | Все результаты владеют строкой; Cap’n Proto reader копирует текст в String |\n\n';
same+='Все кодеки получают одно кольцо из 256 записей; значения и строки побайтно проверяются. Совпадают логические данные и владение, а не бинарные кадры или модель эволюции схем. Для Protobuf типы u8/u16 отсутствуют, поэтому их домен проверяется явно. Подготовка исходных объектов исключена, общий consumer и уничтожение результата включены.\n\n';
for(const group of groups)same+='## '+group+'\n\n'+table(baseline.filter(r=>r.workload===group));
same+='[CSV](results/same-structures.csv) · [исходные измерения и CI](results/released/measurements.csv) · [методика](README.md).\n';
await fs.writeFile(path.join(root,'SAME-STRUCTURES.md'),same);
await fs.writeFile(path.join(root,'results/same-structures.csv'),csv(baseline));
let qsl='# Отдельные бенчмарки QSL 0.4.0\n\n'+conditions;
qsl+='Здесь приведены только QSL-схемы: число бит, физическое выравнивание, pointing, BMI2/group, optional-планы и способы хранения строк. Межпротокольное сравнение находится в [отдельном отчёте](SAME-STRUCTURES.md).\n\n';
const qslRows=n.filter(r=>r.family==='QSL');
for(const group of groups)qsl+='## '+group+'\n\n'+table(qslRows.filter(r=>r.workload===group));
qsl+='PS в данном наборе различает все суммы длин 1, 2 и 4 байта; это свойство выбранной схемы. Отсутствие аллокации у StackString требует объекта 1032 байта. Вариант BMI2 проверен наличием PEXT в исполняемом файле, но ускорение относительно scalar compact определяется измерением. Разницы в несколько процентов при дрейфе или пересечении CI не доказывают превосходство.\n\n';
qsl+='[Все режимы Dyn](DYN.md) · [CSV QSL](results/qsl-only.csv) · [аллокации](results/released/profiles.json).\n';
await fs.writeFile(path.join(root,'QSL-ONLY.md'),qsl);
await fs.writeFile(path.join(root,'results/qsl-only.csv'),csv(qslRows));

// Dyn is a separately built and separately timed executable, sharing the exact fixtures/consumer.
try {
 const env=await read('results/dyn/environment.json');
 let runs=await read('results/dyn/runs.json');if(!Array.isArray(runs))runs=[runs];
 if(runs.length!==2||runs.some(r=>r.exitCode!==0))throw Error('Two complete Dyn passes required');
 const metadata=await read('results/dyn/profiles.json');
 for(const [relative,hash]of Object.entries(env.sources)){
  const actual=createHash('sha256').update(await fs.readFile(path.resolve(root,'dyn',relative.replaceAll('\\','/')))).digest('hex').toUpperCase();
  if(actual!==hash)throw Error('Changed Dyn source: '+relative);
 }
 if(createHash('sha256').update(await fs.readFile(env.executable)).digest('hex').toUpperCase()!==env.executableSha256)throw Error('Changed Dyn executable');
 const maps=[];
 for(const run of runs){
  const folder=path.join(root,'results/dyn',run.run,'criterion'),map=new Map();
  for(const entry of await fs.readdir(folder,{withFileTypes:true})){
   if(!entry.isDirectory())continue;
   const current=path.join(folder,entry.name,'new');
   const b=JSON.parse(await fs.readFile(path.join(current,'benchmark.json'),'utf8'));
   const e=JSON.parse(await fs.readFile(path.join(current,'estimates.json'),'utf8'));
   map.set(b.full_id,e);
  }
  if(map.size!==60)throw Error('Expected 60 Dyn cases, received '+map.size);
  maps.push(map);
 }
 const rows=[];
 for(const [id,a]of maps[0]){
  const b=maps[1].get(id);if(!b)throw Error('Missing Dyn reverse case '+id);
  const [workload,operation,codec='consumer_only']=id.split('/');
  const p=metadata.profiles.find(p=>p.workload===workload&&p.codec===codec&&(operation==='write_prepared'?p.write_measured:p.read_mode===operation));
  if(operation!=='consume_control'&&!p)throw Error('Missing Dyn metadata '+id);
  const x=a.median.point_estimate,y=b.median.point_estimate;
  rows.push({id,workload,operation,codec,median1_ns:x,median2_ns:y,mean_of_medians_ns:(x+y)/2,
   run1_ci99_low:a.median.confidence_interval.lower_bound,run1_ci99_high:a.median.confidence_interval.upper_bound,
   run2_ci99_low:b.median.confidence_interval.lower_bound,run2_ci99_high:b.median.confidence_interval.upper_bound,
   drift_percent:200*Math.abs(x-y)/(x+y),wire_mean:p?.wire_mean??null,wire_min:p?.wire_min??null,wire_max:p?.wire_max??null,
   allocations:p?(operation==='write_prepared'?p.write_allocations:p.read_allocations):null});
 }
 await fs.writeFile(path.join(root,'results/dyn/summary.json'),JSON.stringify({environment:env,runs,measurements:rows,profiles:metadata,sourceIntegrity:'unchanged'},null,2));
 await fs.writeFile(path.join(root,'results/dyn/measurements.csv'),csv(rows));
 let doc='# Dyn v1: все режимы QSL 0.4.0\n\n'+conditions;
 if(env.resumeCpuLimitPercent>20)doc+='**Условия второго прохода отличаются:** перед ним фон CPU вырос до '+nf(runs[1].preRunTotalCpuPercent)+'%. После ожидания возобновлён запуск на том же закреплённом ядре с явно записанным повышенным пределом фоновой нагрузки. Эта пара не является двумя проходами на свободном компьютере; близкие результаты следует считать неразличимыми без повторного тихого измерения. Все исходные медианы, CI и нагрузка сохранены.\n\n';
 doc+='Отдельный release-бинарник и два отдельных прохода: **60 случаев × 2**, 32 профиля, включая owned/borrowed. Измерены все семь разрешённых сочетаний режима и ширины: line 16/32/64; index(dynamic) 32/64; index(strict) 32/64. Для четырёх индексных режимов добавлена числовая схема compact. index16 и compact+line не поддерживаются анализатором QSL 0.4.0; они не выдаются за измеренные случаи.\n\n';
 doc+='Те же numeric, optionals и text_1024, что в межпротокольном сравнении. Теги явные, revision=1, MAX_DATA_SIZE=8192. Все чтения checked: заголовок, границы, значения, UTF-8 и обязательные поля проверяются кодеком; полный consumer включён. Подготовка источников исключена.\n\n';
 doc+='Временная память передаётся явно: line использует u64 на запись, индекс — u32. Локальные массивы scratch имеют ёмкость четыре записи для чисел/optionals и две для текста; их инициализация входит в измеренное чтение. Размеры массивов: 32/16 байт для четырёх записей line/index, 16/8 байт для двух. Heap-выделение scratch не выполняется. borrowed-текст имеет отдельный контракт и не ранжируется с owned.\n\n';
 const numericRead=codec=>rows.find(r=>r.workload==='numeric'&&r.operation==='read_owned'&&r.codec===codec);
 const line16=numericRead('dyn_line_16');
 const line64=numericRead('dyn_line_64');
 const dynamic32=numericRead('dyn_index_dynamic_32');
 const strict32=numericRead('dyn_index_strict_32');
 const compact32=numericRead('dyn_index_dynamic_32_compact');
 doc+='## Результат на четырёх u32\n\n';
 doc+='- `line16` использует '+line16.wire_mean+' B и читает запись за '+nf(line16.mean_of_medians_ns)+' нс. У `line64` кадр занимает '+line64.wire_mean+' B, а чтение — '+nf(line64.mean_of_medians_ns)+' нс.\n';
 doc+='- Оба 32-битных индекса занимают '+dynamic32.wire_mean+' B. Чтение в strict заняло '+nf(strict32.mean_of_medians_ns)+' нс, в dynamic — '+nf(dynamic32.mean_of_medians_ns)+' нс.\n';
 doc+='- `compact` уменьшил кадр dynamic32 до '+compact32.wire_mean+' B. Чтение заняло '+nf(compact32.mean_of_medians_ns)+' нс.\n\n';
 doc+='У чисел и опциональных чисел чтение обходится без heap-аллокаций. Чтение строки 1024 B с владением выделяет память один раз. При заимствовании строки аллокаций нет. Счётчики по всем операциям есть в CSV.\n\n';
 for(const group of ['numeric','optionals','text_1024'])doc+='## '+group+'\n\n'+table(rows.filter(r=>r.workload===group&&r.operation!=='consume_control'));
 doc+='## Проверка и ограничения\n\nВсе 256 записей проверены для каждого профиля. Тесты отклоняют усечённые кадры, неверную ревизию, ненулевые reserved, другой режим/ширину и недостаточный scratch. Тесты проверки не входят во время замера. Line хранит tag+Span возле каждого значения; индексные режимы имеют таблицу и конечное смещение. strict не обязан уменьшать кадр: полная ширина индексной записи такая же. Сравнение с обычным QSL взято из другого прохода в тот же день; для точных малых различий учитывайте фоновую нагрузку.\n\n';
 doc+='Строк с дрейфом >10%: '+rows.filter(r=>r.drift_percent>10).length+'. Фон вне процесса: '+runs.map(r=>nf(r.otherSystemCpuCoreEquivalent)+' ядра').join(' / ')+'.\n\n';
 doc+='[CSV и CI](results/dyn/measurements.csv) · [JSON](results/dyn/summary.json) · [аллокации](results/dyn/profiles.json) · [схемы](dyn/schemas/records.qsl) · [k6](K6.md).\n';
 await fs.writeFile(path.join(root,'DYN.md'),doc);
 console.log('Dyn report:',rows.length,'measurements');
}catch(e){if(e.code!=='ENOENT')throw e;console.log('Dyn results not complete yet');}
console.log('Separate baseline and QSL reports generated');
