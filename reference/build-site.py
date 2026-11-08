from pathlib import Path
import json,html,re,sys
root=Path(__file__).resolve().parents[1];repo=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else root;docs=repo/'docs'
words=json.loads((docs/'data/full-words.json').read_text(encoding='utf-8'))
for k,v in words.items():
 if len(v)==1:words[k]=v*2
 assert len(words[k])==2,k
(docs/'data/full-words.json').write_text(json.dumps(words,ensure_ascii=False,indent=2),encoding='utf-8',newline='\n')
def t(k):return html.escape(words[k][0])
reference_base=docs
def inline(s):
 s=html.escape(s);s=re.sub(r'`([^`]+)`',r'<code>\1</code>',s);s=re.sub(r'\*\*(.+?)\*\*',r'<strong>\1</strong>',s)
 def link(m):
  href=m[2]
  if not href.startswith(('https:','http:','#')) and href.endswith('.md'):
   target=(reference_base/href).resolve()
   if target.is_relative_to(docs.resolve()) and target.exists():href=href[:-3]+'.html'
  return '<a href="'+href+'">'+m[1]+'</a>'
 return re.sub(r'\[([^\]]+)\]\(([^)]+)\)',link,s)
def render(source,prefix):
 lines=source.splitlines();out=[];toc=[];i=0;section=0
 while i<len(lines):
  line=lines[i]
  if not line.strip():i+=1;continue
  if line.startswith('```'):
   code=[];i+=1
   while i<len(lines)and not lines[i].startswith('```'):code.append(lines[i]);i+=1
   out.append('<pre><code>'+html.escape('\n'.join(code))+'</code></pre>');i+=1;continue
  h=re.match(r'^(#{1,6})\s+(.+)',line)
  if h:
   section+=1;level=max(2,len(h[1]));id=f'{prefix}-{section}';out.append(f'<h{level} id="{id}">'+inline(h[2])+f'</h{level}>')
   if len(h[1])==2:toc.append((id,h[2]))
   i+=1;continue
  if line.startswith('|'):
   rows=[]
   while i<len(lines)and lines[i].startswith('|'):
    if not re.fullmatch(r'[|\s:\-]+',lines[i]):rows.append([inline(x.strip())for x in lines[i].strip('|').split('|')])
    i+=1
   out.append('<div class="table-scroll"><table><thead><tr>'+''.join('<th>'+x+'</th>'for x in rows[0])+'</tr></thead><tbody>'+''.join('<tr>'+''.join('<td>'+x+'</td>'for x in row)+'</tr>'for row in rows[1:])+'</tbody></table></div>');continue
  if re.match(r'^\d+\. ',line):
   items=[]
   while i<len(lines)and re.match(r'^\d+\. ',lines[i]):items.append(re.sub(r'^\d+\. ','',lines[i]));i+=1
   out.append('<ol>'+''.join('<li>'+inline(x)+'</li>'for x in items)+'</ol>');continue
  if line.startswith('- '):
   items=[]
   while i<len(lines)and lines[i].startswith('- '):items.append(lines[i][2:]);i+=1
   out.append('<ul>'+''.join('<li>'+inline(x)+'</li>'for x in items)+'</ul>');continue
  if line.startswith('> '):out.append('<blockquote>'+inline(line[2:])+'</blockquote>');i+=1;continue
  paragraph=[line];i+=1
  while i<len(lines)and lines[i].strip()and not lines[i].startswith(('#','```','|','> ','- '))and not re.match(r'^\d+\. ',lines[i]):paragraph.append(lines[i]);i+=1
  out.append('<p>'+inline(' '.join(paragraph))+'</p>')
 return '\n'.join(out),toc

import hashlib,runpy
complete_home=runpy.run_path(str(repo/"reference/complete-manual.py"))["complete_home"]
compat_version=hashlib.sha256((docs/"assets/manual.css").read_bytes()).hexdigest()[:10]
css_version=hashlib.sha256((docs/'assets/full.css').read_bytes()).hexdigest()[:10]
js_version=hashlib.sha256((docs/'assets/manual.js').read_bytes()).hexdigest()[:10]
data_version=hashlib.sha256(b''.join(p.read_bytes() for p in [docs/'data/api.json',docs/'data/strings-current.json'] if p.exists())).hexdigest()[:12]
def page_head(lang,title,depth=''):
 return '<!doctype html><html lang="'+lang+'" data-generation="3"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta name="theme-color" content="#ffffff"><title>'+html.escape(title)+'</title><link rel="stylesheet" href="'+depth+'assets/full.css?v='+css_version+'"><link rel="stylesheet" href="'+depth+'assets/manual.css?v='+compat_version+'"></head><body data-data-version="'+data_version+'">'
def home(lang):
 en=lang=='en';L=lambda ru,en_text:en_text if en else ru;suffix='en' if en else 'ru';other='index.html' if en else 'en.html'
 benchmark=json.loads((docs/'data/strings-current.json').read_text(encoding='utf-8')) if (docs/'data/strings-current.json').exists() else {'rows':[],'status':{},'conditions':{}}
 fmt=lambda n:format(n,'.3f').replace('.',',' if not en else '.')
 static_rows=''.join('<tr><td>'+html.escape(r['label'])+' · '+str(r['useful_bytes'])+' B</td><td class="num">'+fmt(r['ns'])+'</td><td class="num">'+fmt(r['low'])+'–'+fmt(r['high'])+'</td><td class="num">'+fmt(r['P'])+'</td><td class="num">'+str(r['wire_bytes'])+' B</td></tr>'for r in benchmark['rows'])
 links=[('start',L('Начало','Start')),('architecture',L('Архитектура','Architecture')),('strings',L('Строки и память','Strings and memory')),('optional',L('Optional-поля','Optional fields')),('measurements',L('Новые замеры','Current measurements')),('reading',L('Руководства','Guides')),('api',L('Поиск API','API search'))]
 navigation=''.join('<a href="#'+a+'">'+b+'</a>'for a,b in links)
 reading=[('quickstart',L('Быстрый старт','Quick start'),L('Схема, build.rs и первый кодек.','Schema, build.rs and the first codec.')),('guide',L('Полное руководство','Complete guide'),L('Язык, атрибуты, владение и ограничения.','Language, attributes, ownership and limits.')),('architecture',L('Устройство компилятора','Compiler architecture'),L('Core, сборщик, парсер и Rust-генератор.','Core, schema builder, frontend and Rust backend.')),('optional-modes',L('BM, PS и OT','BM, PS and OT'),L('Маска, индекс длины и таблица читателей.','Presence mask, length index and reader table.')),('safety',L('Проверяемый вход','Checked input'),L('Границы, UTF-8, распаковка и бюджеты.','Bounds, UTF-8, decompression and budgets.')),('custom-types',L('Пользовательские типы','Custom types'),L('Примитивы, макросы и ABI-контракты.','Primitives, macros and ABI contracts.')),('schema-hashes',L('Идентичность схем','Schema identity'),L('Строгий и лёгкий хэши, wire-атрибуты.','Strict/light hashes and wire-affecting attributes.')),('migration-0.3.0',L('Изменения текущей ветки','Current branch changes'),L('Перенос в qsl-core и строковые представления.','qsl-core extraction and string representations.'))]
 read_html=''.join('<a href="'+name+'.'+suffix+'.html">'+title+'<small>'+detail+'</small></a>'for name,title,detail in reading)
 h=page_head(lang,'QSL · '+L('Интерактивная документация','Interactive documentation'))
 h+=f'''<a class="skip-link" href="#main">{L('К содержанию','Skip to content')}</a><header class="top"><a class="brand" href="{('en.html' if en else 'index.html')}">QSL</a><span class="top-note">{L('Схемы и бинарные кодеки для Rust','Schemas and binary codecs for Rust')}</span><nav aria-label="Language"><a href="index.html"{('' if en else ' aria-current="page"')}>RU</a><a href="en.html"{(' aria-current="page"' if en else '')}>EN</a><a href="https://github.com/zadr0/QSL">GitHub ↗</a></nav></header><div class="shell"><aside class="sidebar"><strong>{L('Документация','Documentation')}</strong>{navigation}<strong>{L('Материалы','Resources')}</strong><a href="API.{suffix}.html">{L('Полный API','Complete API')}</a><a href="features.{suffix}.html">{L('Исторические замеры','Historical measurements')}</a></aside><main id="main" class="content">
<section id="start" class="hero"><div class="eyebrow">QSL / RUST / MAIN</div><h1>{L('Обмен заранее описанными данными','Exchange of data described in advance')}</h1><p class="lead">{L('QSL анализирует схему при сборке и генерирует конкретные Rust-структуры и кодеки. Типы, границы, порядок полей и выбор памяти задаются явно. Во время обмена готовый код читает и записывает байты.','QSL analyzes a schema at build time and generates concrete Rust structures and codecs. Types, bounds, field order and storage are explicit. Generated code reads and writes bytes at runtime.')}</p><div class="badges"><span class="badge">Rust 1.98+</span><span class="badge">MIT</span><span class="badge">RU / EN</span><span class="badge">{L('Текущая ветка: qsl-core','Current branch: qsl-core')}</span></div><p>{L('Подходит для согласованных внутренних протоколов, телеметрии и повторяемых записей. Приложение отвечает за транспорт, TLS и согласование версий.','Designed for agreed internal protocols, telemetry and repeated records. Applications own transport, TLS and version negotiation.')}</p><div class="actions"><a class="primary" href="quickstart.{suffix}.html">{L('Начать работу','Get started')}</a><a href="#strings">{L('Исследовать память','Explore storage')}</a><a href="https://github.com/zadr0/QSL/releases">{L('Выпуски и пакеты','Releases and packages')}</a></div></section>
<section class="chapter" id="architecture"><header><h2>{L('Одна схема, разделённые обязанности','One schema, separate responsibilities')}</h2><span class="section-label">01 / ARCHITECTURE</span></header><p>{L('Общие контракты и исполнение вынесены в qsl-core. Lexer собирает и проверяет схемы, interpreter генерирует Rust. Фасад qsl сохраняет пути ::qsl в готовом коде.','Shared contracts and runtime execution belong to qsl-core. The lexer collects and validates schemas; the interpreter generates Rust. The qsl facade preserves ::qsl paths in generated code.')}</p><div class="pipeline"><a href="https://github.com/zadr0/QSL/tree/main/qsl-core"><code>qsl-core</code><small>{L('Типы, ошибки, I/O, строки, буферы','Types, errors, I/O, strings, buffers')}</small></a><a href="https://github.com/zadr0/QSL/tree/main/qsl-lexer"><code>qsl-lexer</code><small>{L('QSL / DEQ → проверенные схемы','QSL / DEQ → validated schemas')}</small></a><a href="https://github.com/zadr0/QSL/tree/main/qsl-interpreter"><code>qsl-interpreter</code><small>{L('Схемы → исходный Rust-код','Schemas → generated Rust source')}</small></a><a href="https://github.com/zadr0/QSL/tree/main/qsl"><code>qsl</code><small>{L('Фасад приложения и совместимые пути','Application facade and compatible paths')}</small></a></div><div class="grid-2"><div class="panel"><h3>{L('Схема','Schema')}</h3><pre><code>&amp;Message[safety,opt_table] {{
    u32 id;
    str(1..255) text;
    u16? code;
}}</code></pre><p class="note">{L('Обязательные поля всегда стоят перед опциональными. Атрибуты выбираются отдельно.','Required fields always precede optional fields. Attributes are independent choices.')}</p></div><div class="panel"><h3>{L('Генерация при сборке','Build-time generation')}</h3><pre><code>use qsl_interpreter::prelude::*;
let builder = build_qsl_schem("schemas")?;
let rust = builder.use_interpreter()?;</code></pre><p class="note">{L('Сборщик сохраняет схемы. В будущем другой backend сможет использовать тот же контракт SchemaBackend; LLVM пока не реализован.','The builder retains the schema collection. Another backend can implement SchemaBackend; LLVM is not implemented.')}</p></div></div></section>
<section class="chapter" id="strings"><header><h2>{L('UTF-8: тип, память и режим чтения','UTF-8: type, storage and read mode')}</h2><span class="section-label">02 / STORAGE</span></header><p>{L('Переменные строки представлены только String, &str, StackString и InlineString. New/reuse — режим работы с назначением. Guard и байтовый срез не являются отдельными строковыми реализациями QSL.','Variable strings use String, &str, StackString and InlineString only. New/reuse describes destination handling. Guard and byte slices are not QSL string representations.')}</p><div class="grid-2"><div class="panel"><div class="controls"><label>{L('Представление','Representation')}<select id="string-kind"><option>String</option><option>&amp;str</option><option>StackString</option><option>InlineString</option></select></label><label>{L('Максимум UTF-8, B','UTF-8 maximum, B')}<input id="string-bound" type="number" min="1" max="65535" value="255"></label><label>STACK_LIMIT, B<input id="stack-budget" type="number" min="0" max="65535" value="256"></label><label class="check"><input type="checkbox" id="string-reuse" checked>reuse</label><label class="full-width">{L('Текст для проверки длины','Text for byte-length inspection')}<textarea id="string-text">Рад, так Рад!</textarea></label></div><p id="memory-status" class="status" aria-live="polite"></p><pre><code id="string-schema"></code></pre></div><div class="panel"><h3>{L('План хранения · x86-64','Storage plan · x86-64')}</h3><div class="stats"><div class="stat"><small>{L('Rust-тип','Rust type')}</small><strong id="memory-type">String</strong></div><div class="stat"><small>{L('Объект','Object')}</small><strong id="memory-object">24 B</strong></div><div class="stat"><small>{L('Полезные байты','Useful bytes')}</small><strong id="memory-payload">255 B</strong></div></div><p id="memory-mode" class="note"></p><p class="note">{L('Это модель раскладки по объявленной ёмкости, не измерение RSS. Ёмкость String может быть больше фактической длины; накладные расходы аллокатора сюда не входят. Окончательный ABI определяет rustc.','This models declared capacity, not measured RSS. String capacity may exceed its length; allocator overhead is excluded. rustc determines the final ABI.')}</p><div class="notice">{L('По умолчанию STACK_LIMIT = 256 B. Превышение лимита переводит обычные inline-строки в String. Для нативного фиксированного поля сохраняется [u8; N]; заимствованная строка требует живого исходного кадра.','STACK_LIMIT defaults to 256 B. Larger ordinary inline strings fall back to String. Fixed native byte fields retain [u8; N]; borrowed text requires the original frame to remain alive.')}</div><a href="migration-0.3.0.{suffix}.html">{L('Контракт строк и перенос API','String contract and API migration')} →</a></div></div></section>
<section class="chapter" id="optional"><header><h2>{L('BM, predict_seq и opt_table','BM, predict_seq and opt_table')}</h2><span class="section-label">03 / OPTIONAL</span></header><div class="grid-3"><div class="card"><span class="number">BM</span><h3>{L('Присутствие','Presence')}</h3><p>{L('Маска только опциональных полей. Ширина 8–128 бит выбирается при анализе; обход читает установленные биты.','A mask of optional fields only. Analysis selects 8–128 bits; traversal visits the set bits.')}</p></div><div class="card"><span class="number">PS</span><h3>{L('Индекс по длине','Length index')}</h3><p>{L('predict_seq использует длину payload для допустимых последовательностей. При BM этот механизм не применяется.','predict_seq derives admissible sequences from payload length. It is inactive with BM.')}</p></div><div class="card"><span class="number">OT</span><h3>{L('Выбор читателя','Reader dispatch')}</h3><p>{L('opt_table связывает проверенный маркер с функцией чтения. Сохраняет формат и сочетается с BM или PS.','opt_table maps a validated marker to a reader. It preserves the format and can combine with BM or PS.')}</p></div></div><div class="panel"><div class="grid-2"><div><pre><code>&amp;Packet[safety,optionals=bitmap,opt_table] {{
    u8 id;   // 9
    u16? a;  // 0x1234
    u8? b;   // 7
}}</code></pre><div class="controls"><label class="check"><input id="bm-a" type="checkbox" checked>a</label><label class="check"><input id="bm-b" type="checkbox" checked>b</label><label class="check"><input id="bm-ot" type="checkbox" checked>opt_table</label><label class="check"><input id="bm-invalid" type="checkbox">{L('Лишний бит','Spare bit')}</label></div><p class="note">{L('Маркер a = 1 находится в старшем бите. Чтение trailing_zeros даёт порядок b → a; writer использует тот же порядок.','Marker a = 1 occupies the most significant bit. trailing_zeros reads b → a; the writer uses the same order.')}</p></div><div><div class="stats"><div class="stat"><small>{L('Маска','Mask')}</small><strong id="bm-mask">11000000</strong></div><div class="stat"><small>{L('Маркеры','Markers')}</small><strong id="bm-order">2 → 1</strong></div><div class="stat"><small>{L('Читатели','Readers')}</small><strong id="bm-readers">OT[1] → OT[0]</strong></div></div><div id="bm-wire" class="wire" aria-label="Wire bytes"><span class="metadata">C0</span><span>09</span><span>07</span><span>34</span><span>12</span></div><p id="bm-status" class="status" aria-live="polite"></p><p class="note">{L('Синий байт — метаданные. Это модель конкретного формата, проверенная в Rust; она не симулирует время исполнения.','The blue byte is metadata. This models a concrete format verified in Rust; it does not simulate execution time.')}</p></div></div></div><p class="notice">{L('QPS — отдельная обратимая упаковка чисел по операндам и диапазонам. Она не предсказывает наличие optional-полей. Сжатие UTF-8 через decompress=true использует самостоятельный кодек.','QPS is separate lossless operand/range packing for numbers. It does not predict optional-field presence. UTF-8 compression with decompress=true uses its own codec.')}</p></section>
<section class="chapter" id="measurements"><header><h2>{L('Свежие измерения строк','Current string measurements')}</h2><span class="section-label">04 / CRITERION</span></header><p>{L('Девять сравнений чтения: new/reuse и &str. Основной набор содержит 1024 символа кириллицы = 2048 полезных байт. Контроль String использует 255 ASCII-символов = 255 байт.','Nine decoding cases compare new/reuse and &str. The main set contains 1024 Cyrillic scalars = 2048 useful bytes. The String control uses 255 ASCII scalars = 255 bytes.')}</p><div class="notice"><strong>P = M / T</strong> · {L('M — исходные полезные байты; T — наносекунды на одну строку.','M is original useful bytes; T is nanoseconds per string.')}</div><div class="panel"><p id="bench-status" class="status" aria-live="polite">{html.escape(benchmark.get('status',{}).get(lang,L('Результаты ещё не загружены.','Results are not loaded yet.')))}</p><p id="bench-conditions" class="note">{html.escape(benchmark.get("conditions",{}).get(lang,""))}</p><div id="bench-results"{("" if static_rows else " hidden")}><div class="controls"><label>{L('Показатель','Metric')}<select id="metric"><option value="T">T, ns</option><option value="P">P, B/ns</option></select></label><span id="bars-unit" class="note"></span></div><div class="plot-scroll"><svg id="bench-bars" class="plot" role="img" aria-label="Criterion comparison"></svg></div><div class="table-scroll"><table><thead><tr><th>{L('Случай','Case')}</th><th>T, ns</th><th>99% CI, ns</th><th>P, B/ns</th><th>Wire</th></tr></thead><tbody id="bench-body">{static_rows}</tbody></table></div><h3>{L('Кардиограмма выбранного случая','Selected case waveform')}</h3><p class="note">{L('Выберите строку таблицы. Каждая точка — исходная нормированная выборка Criterion, без сглаживания.','Select a table row. Each point is an original normalized Criterion sample, without smoothing.')}</p><p id="wave-label" class="note"></p><div class="plot-scroll"><svg id="bench-wave" class="plot" role="img" aria-label="Criterion sample series"></svg></div></div><p class="actions"><a href="data/strings-current.json">JSON</a><a href="performance.{suffix}.html">{L('Методика и ограничения','Methodology and limitations')}</a><a href="features.{suffix}.html">{L('Архив прежних профилей','Earlier profile archive')}</a></p></div></section>
<section class="chapter" id="reading"><header><h2>{L('Руководства и контракты','Guides and contracts')}</h2><span class="section-label">05 / REFERENCE</span></header><div class="reading-list">{read_html}</div></section>
<section class="chapter" id="api"><header><h2>{L('Поиск по текущему API','Search the current API')}</h2><span class="section-label">06 / SOURCE</span></header><p>{L('Индекс строится из исходников core, lexer, interpreter и фасада. Ссылки ведут к объявлению в текущей ветке.','The index is built from core, lexer, interpreter and facade source. Links point to declarations on the current branch.')}</p><div class="panel"><div class="controls"><label class="full-width">{L('Имя, модуль или сигнатура','Name, module or signature')}<input id="api-query" type="search" placeholder="StackString · deserialize_into · QslError"></label></div><p id="api-status" class="status"></p><div id="api-results" class="api-list"></div><p><a href="API.{suffix}.html">{L('Полный статический индекс','Complete static index')} →</a></p></div></section>
<footer class="footer">QSL · MIT · <a href="https://github.com/zadr0/QSL">GitHub</a> · <a href="https://github.com/zadr0/QSL-Extension">Zed</a> · {L('Документация текущей ветки; измерения привязаны к сохранённым исходникам.','Current-branch documentation; measurements identify their captured sources.')}</footer></main></div><script src="assets/manual.js?v={js_version}" defer></script></body></html>'''
 return complete_home(h,lang,repo,render)
for lang,path in [('ru','index.html'),('en','en.html')]:
 (docs/path).write_text(home(lang),encoding='utf-8',newline='\n')
# Both offline language editions embed their data and scripts.
for lang,filename in [('ru','QSL_standalone_RU_EN.html'),('en','QSL_standalone_EN.html')]:
 page=home(lang).replace('<link rel="stylesheet" href="assets/full.css?v='+css_version+'">','<style>'+(docs/'assets/full.css').read_text(encoding='utf-8')+'</style>')
 for name in ['manual.css','explorers.css']:
  page=re.sub(r'<link rel="stylesheet" href="assets/'+re.escape(name)+r'\?v=[^"]+">',lambda m:'<style>'+(docs/'assets'/name).read_text(encoding='utf-8')+'</style>',page)
 page=re.sub(r'<script src="assets/workshop.js\?v=[^"]+" defer></script>',lambda m:'<script>'+(docs/'assets/workshop.js').read_text(encoding='utf-8')+'</script>',page)
 page=page.replace('href="index.html"','href="QSL_standalone_RU_EN.html"').replace('href="en.html"','href="QSL_standalone_EN.html"')
 for key,name in [('current-api-data','api.json'),('current-bench-data','strings-current.json')]:
  file=docs/'data'/name
  if file.exists():
   data=json.dumps(json.loads(file.read_text(encoding='utf-8')),ensure_ascii=False,separators=(',',':')).replace('<',chr(92)+'u003c')
   page=page.replace('</body>','<script type="application/json" id="'+key+'">'+data+'</script></body>')
 page=page.replace('<script src="assets/manual.js?v='+js_version+'" defer></script>','')
 page=page.replace('</body>','<script>'+(docs/'assets/manual.js').read_text(encoding='utf-8')+'</script></body>')
 (docs/filename).write_text(page,encoding='utf-8',newline='\n')
# Render current prose with one shared, strict stylesheet and visible language links.
names=['guide','features','architecture','API','performance','opt-table-control','mathematics','safety','schema-hashes','examples','optional-modes','quickstart','migration-0.2.0','migration-0.3.0','custom-types','SEO']
for lang in ['ru','en']:
 for name in names:
  source=docs/f'{name}.{lang}.md'
  if not source.exists():continue
  reference_base=source.parent
  content,_=render(source.read_text(encoding='utf-8'),name+'-'+lang);content=content.replace('<h2','<h1',1).replace('</h2>','</h1>',1)
  other='en' if lang=='ru' else 'ru';entry='en.html' if lang=='en' else 'index.html'
  navigation='<nav class="doc-header"><a href="'+entry+'">QSL / '+('Documentation' if lang=='en' else 'Документация')+'</a><a href="'+name+'.'+other+'.html">'+('Русский' if lang=='en' else 'English')+'</a></nav>'
  (docs/f'{name}.{lang}.html').write_text(page_head(lang,'QSL · '+name)+'<main class="book">'+navigation+content+'</main></body></html>',encoding='utf-8',newline='\n')
# Previously released notes and reports retain their stated version.
for lang in ['ru','en']:
 for folder in ['releases','modernization-2026-11-06']:
  for source in (docs/folder).glob('*.'+lang+'.md'):
   reference_base=source.parent
   content,_=render(source.read_text(encoding='utf-8'),folder+'-'+lang);content=content.replace('<h2','<h1',1).replace('</h2>','</h1>',1)
   source.with_suffix('.html').write_text(page_head(lang,'QSL · '+source.stem,'../')+'<main class="book"><a href="../'+('en.html' if lang=='en' else 'index.html')+'">QSL</a>'+content+'</main></body></html>',encoding='utf-8',newline='\n')
for lang in ['ru','en']:
 for source in (docs/'attributes'/lang).glob('*.md'):
  reference_base=source.parent
  content,_=render(source.read_text(encoding='utf-8'),'attribute-'+source.stem+'-'+lang)
  content=content.replace('<h2','<h1',1).replace('</h2>','</h1>',1)
  entry='en.html' if lang=='en' else 'index.html';other='ru' if lang=='en' else 'en'
  nav='<nav class="doc-header"><a href="../../'+entry+'">QSL / '+('Attributes' if lang=='en' else 'Атрибуты')+'</a><a href="../'+other+'/'+source.stem+'.html">'+('Русский' if lang=='en' else 'English')+'</a></nav>'
  source.with_suffix('.html').write_text(page_head(lang,'QSL · '+source.stem,'../../')+'<main class="book">'+nav+content+'</main></body></html>',encoding='utf-8',newline='\n')
runpy.run_path(str(repo/'reference/apply-seo.py'),run_name='__main__')
print('Built current RU/EN manual, strict shared typography and focused interactive tools.')
