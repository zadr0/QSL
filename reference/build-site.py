from pathlib import Path
import json,html,re,sys
root=Path.cwd();repo=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else root/'.optimization/qsl-public-docs';docs=repo/'docs'
words=json.loads((docs/'data/full-words.json').read_text(encoding='utf-8'))
for k,v in words.items():
 if len(v)==1:words[k]=v*2
 assert len(words[k])==2,k
(docs/'data/full-words.json').write_text(json.dumps(words,ensure_ascii=False,indent=2),encoding='utf-8',newline='\n')
def t(k):return html.escape(words[k][0])
def inline(s):
 s=html.escape(s);s=re.sub(r'`([^`]+)`',r'<code>\1</code>',s);s=re.sub(r'\*\*(.+?)\*\*',r'<strong>\1</strong>',s)
 def link(m):
  href=m[2]
  if re.search(r'(guide|features|architecture|API|performance|opt-table-control|mathematics|safety|schema-hashes|examples|optional-modes|MODERNIZATION)\.(ru|en)\.md$',href):href=href[:-3]+'.html'
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
books=[];nav=[]
for lang in ['ru','en']:
 content,toc=render((docs/f'guide.{lang}.md').read_text(encoding='utf-8'),'guide-'+lang)
 books.append(f'<article class="book locale" data-locale="{lang}"'+(' hidden'if lang=='en'else'')+'>'+content+'</article>')
 nav.append(f'<div class="doc-nav locale" data-locale="{lang}"'+(' hidden'if lang=='en'else'')+'>'+''.join(f'<a href="#{id}">{html.escape(title)}</a>'for id,title in toc)+'</div>')
head='<!doctype html><html lang="ru"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta name="theme-color" content="#f4f5f7"><title>QSL · '+t('project')+'</title><link rel="stylesheet" href="assets/full.css"><link rel="stylesheet" href="assets/explorers.css"></head><body>'
body=f'''<header class="top"><a class="brand" href="#overview">QSL</a><small data-t="project">{t('project')}</small><span class="spacer"></span><button data-lang="ru" aria-pressed="true">RU</button><button data-lang="en" aria-pressed="false">EN</button></header><div class="shell"><aside class="sidebar"><a href="#overview" data-t="overview">{t('overview')}</a><a href="#measurements" data-t="measurements">{t('measurements')}</a><a href="#qps-demo" data-t="quantDemo">{t('quantDemo')}</a><a href="#event-demo" data-t="eventDemo">{t('eventDemo')}</a><a href="#api" data-t="api">{t('api')}</a><a href="#history" data-t="history">{t('history')}</a><div class="sep"></div><h2 data-t="guide">{t('guide')}</h2>{''.join(nav)}</aside><main class="content">
<section id="overview" class="hero"><div class="eyebrow">QSL / RUST</div><h1 data-t="hero">{t('hero')}</h1><p data-t="lead">{t('lead')}</p><div class="actions"><a data-doc="guide" href="guide.ru.html" data-t="guide">{t('guide')}</a><a data-doc="features" href="features.ru.html" data-t="download">{t('download')}</a><a href="../SOURCE_AUDIT.md" data-t="audit">{t('audit')}</a></div><div class="cards"><div class="card"><strong data-t="cardSchema">{t('cardSchema')}</strong><p data-t="cardSchemaText">{t('cardSchemaText')}</p></div><div class="card"><strong data-t="cardMemory">{t('cardMemory')}</strong><p data-t="cardMemoryText">{t('cardMemoryText')}</p></div><div class="card"><strong data-t="cardCodec">{t('cardCodec')}</strong><p data-t="cardCodecText">{t('cardCodecText')}</p></div></div><pre><code>&amp;Message[safety,opt_table] {{
    u32 id;
    str(0..4096) text|binary_opt=guard|;
    u16? code;
}}</code></pre></section>
<section id="measurements"><h2 data-t="measurements">{t('measurements')}</h2><div class="facts"><span id="run-status"></span><span data-t="conditions">{t('conditions')}</span><span id="run-memory"></span></div><p id="hardware" class="note"></p><p data-t="Pdefinition">{t('Pdefinition')}</p><p class="notice" data-t="compareNote">{t('compareNote')}</p>
<div class="panel"><div class="controls"><label><span data-t="feature">{t('feature')}</span><select id="feature-group"></select></label><label><span data-t="operation">{t('operation')}</span><select id="operation"><option value="read" data-t="read">{t('read')}</option><option value="write" data-t="write">{t('write')}</option></select></label><label><span data-t="metric">{t('metric')}</span><select id="metric"><option value="P" data-t="Pchoice">{t('Pchoice')}</option><option value="T" data-t="Tchoice">{t('Tchoice')}</option><option value="wire" data-t="Wchoice">{t('Wchoice')}</option></select></label></div><p id="group-note" class="note"></p><p id="plot-unit" class="note"></p><div class="plot-shell"><svg id="comparison" class="plot" role="img"></svg></div><div class="table-scroll"><table><thead><tr><th data-t="mode">{t('mode')}</th><th data-t="M">{t('M')}</th><th data-t="wire">{t('wire')}</th><th data-t="time">{t('time')}</th><th data-t="P">{t('P')}</th><th data-t="PCI">{t('PCI')}</th></tr></thead><tbody id="result-body"></tbody></table></div></div>
<div class="panel"><h3 data-t="wave">{t('wave')}</h3><p class="note" data-t="waveExplain">{t('waveExplain')}</p><div class="controls"><label><span data-t="case">{t('case')}</span><select id="wave-case"></select></label><label><span data-t="smooth">{t('smooth')}</span><select id="smoothing"><option value="1" data-t="none">{t('none')}</option><option value="5">5</option><option value="15">15</option></select></label></div><p id="case-summary" class="note"></p><p id="wave-unit" class="note"></p><div class="plot-shell"><svg id="waveform" class="plot" role="img"></svg></div><h3 data-t="hist">{t('hist')}</h3><div class="plot-shell"><svg id="histogram" class="plot" role="img" aria-label="Criterion sample distribution"></svg></div><a id="case-raw" data-t="rawSamples">{t('rawSamples')}</a></div>
<p class="actions"><a data-doc="features" href="features.ru.html" data-t="download">{t('download')}</a> · <a href="data/features.csv">CSV</a> · <a href="data/features.json">JSON</a> · <a href="../reference/features-source-manifest.json">SHA-256</a></p></section>
<section id="qps-demo" class="panel"><h2 data-t="quantDemo">{t('quantDemo')}</h2><p data-t="qpsNote">{t('qpsNote')}</p><div class="controls"><label><span data-t="text">{t('text')}</span><input id="qps-text" type="text" value="{'A'*128}"></label><label><span data-t="quantum">{t('quantum')}</span><select id="qps-quantum"><option value="2">Q2</option><option value="4">Q4</option><option value="8">Q8</option><option value="16" selected>Q16</option></select></label></div><p class="error" id="qps-error" role="status"></p><div class="kv"><div><span data-t="sourceBytes">{t('sourceBytes')}</span><strong id="qps-source"></strong></div><div><span data-t="frame">{t('frame')}</span><strong id="qps-wire"></strong></div><div><span data-t="qpsMode">{t('qpsMode')}</span><strong id="qps-mode"></strong></div><div><span data-t="originalLength">{t('originalLength')}</span><strong id="qps-original"></strong></div></div><div class="output" id="qps-detail"></div><p class="bytes" id="qps-hex"></p><p class="note" data-t="hexNote">{t('hexNote')}</p></section>
<section id="event-demo" class="panel"><h2 data-t="eventDemo">{t('eventDemo')}</h2><p data-t="eventNote">{t('eventNote')}</p><div class="controls"><label><span data-t="presentA">{t('presentA')}</span><input id="event-a" type="checkbox" checked></label><label><span data-t="presentB">{t('presentB')}</span><input id="event-b" type="checkbox"></label></div><div class="output" id="event-result"></div><div class="kv"><div><span data-t="forced">{t('forced')}</span><strong id="event-forced"></strong></div><div><span data-t="possible">{t('possible')}</span><strong id="event-possible"></strong></div></div></section>
<section id="full-guide"><h2 data-t="guide">{t('guide')}</h2>{''.join(books)}</section>
<section id="api"><h2 data-t="api">{t('api')}</h2><div class="controls"><input id="api-query" type="search" aria-label="API search"></div><p id="api-count" class="note"></p><div id="api-results" class="api-list"></div><p class="note" data-t="sourceNote">{t('sourceNote')}</p><a data-doc="API" href="API.ru.html" data-t="api">{t('api')}</a></section>
<section id="history"><a id="control"></a><h2 data-t="history">{t('history')}</h2><p class="note" data-t="historyNote">{t('historyNote')}</p><p><a data-doc="opt-table-control" href="opt-table-control.ru.html">default / opt_table · N=1/2/3/5/10</a> · <a data-doc="performance" href="performance.ru.html">Scalar QPS / opt_table</a></p></section><footer class="footer">QSL · <a href="https://github.com/zadr0/QSL">GitHub</a> · <a href="https://github.com/zadr0/QSL-Extension">Zed</a></footer></main></div>'''
body=body.replace('<div class="actions"><a data-doc="guide"', '<p data-t="heroScope">'+t('heroScope')+'</p><div class="hero-rail"><code>schema.qsl</code><span aria-hidden="true">→</span><code>qsl_gen.rs</code><span aria-hidden="true">→</span><code>application</code><span aria-hidden="true">→</span><code>frame</code></div><div class="actions"><a href="#layout-lab" data-t="openLab">'+t('openLab')+'</a><a data-doc="guide"',1)
# Interactive, concrete models supplement the measured data without estimating timings.
fragment=(docs/'assets/explorers.html').read_text(encoding='utf-8')
body=body.replace('<section id="measurements">',fragment+'\n<section id="measurements" class="chapter"><div class="section-kicker">04 / CRITERION</div>')
body=body.replace('<a href="#measurements" data-t="measurements">','<a href="#workshop" data-t="workshop">'+t('workshop')+'</a><a href="#layout-lab" data-t="layoutTitle">'+t('layoutTitle')+'</a><a href="#memory-lab" data-t="memoryTitle">'+t('memoryTitle')+'</a><a href="#measurements" data-t="measurements">',1)
presets='<div class="qps-presets"><button type="button" data-preset="repeat" data-t="qpsRepeat"></button><button type="button" data-preset="range" data-t="qpsRange"></button><button type="button" data-preset="utf8" data-t="qpsUtf8"></button></div>'
body=body.replace('<div class="controls"><label><span data-t="text">',presets+'<div class="controls"><label><span data-t="text">')
qps_extra='<div class="operand-group"><label><span data-t="qpsGroup"></span><input id="qps-group" type="range" min="0" max="7" value="0"><output id="qps-group-value"></output></label><h3 data-t="qpsOperand"></h3><p id="qps-operand" class="note"></p><div class="operand-grid"><div><h4 data-t="qpsSourceRank"></h4><div id="qps-ranks" class="operand-cells"></div></div><div><h4 data-t="qpsPackedWord"></h4><div id="qps-word-bytes" class="operand-cells"></div><h4 data-t="qpsRestoredGroup"></h4><div id="qps-restored-bytes" class="operand-cells"></div></div></div><p id="qps-raw-note" class="note" data-t="qpsRawNote"></p></div>'
body=re.sub(r'(<section id="qps-demo".*?)(</section>)',lambda m:m[1]+qps_extra+m[2],body,flags=re.S)
events='<h3 data-t="eventGraph"></h3><div class="event-graph"><div class="event-node" data-event="0"><strong data-t="eventL0"></strong><code>I=0 · ∅</code></div><div class="event-node" data-event="1"><strong data-t="eventL1"></strong><code>I=1 · a / b</code></div><div class="event-node" data-event="2"><strong data-t="eventL2"></strong><code>I=2 · a + b</code></div></div><div id="event-bytes" class="wire-strip"></div><p class="note" data-t="eventWireNote"></p><p class="note" data-t="eventModel"></p>'
body=body.replace('<div class="output" id="event-result">',events+'<div class="output" id="event-result">')
notice='<div class="panel"><p class="locale" data-locale="ru"><strong>Checked-чтение при [safety] · MAX_DATA_SIZE = 65535 B</strong><br>Общий бюджет проверяется до создания значений; длина восстановления QPS — до записи в регион. <a href="safety.ru.html">Контракт и методы</a> · <a href="schema-hashes.ru.html">Строгий и лёгкий хэши</a></p><p class="locale" data-locale="en" hidden><strong>Checked decoding with [safety] · MAX_DATA_SIZE = 65535 B</strong><br>Aggregate data is checked before value construction; QPS original length before restoration. <a href="safety.en.html">Contract and methods</a> · <a href="schema-hashes.en.html">Strict and light identities</a></p></div>'
examples_notice='<div class="panel"><h2 class="locale" data-locale="ru">Запускаемые примеры · 39 операций исходного профиля</h2><h2 class="locale" data-locale="en" hidden>Executable examples · 39 original-profile operations</h2><p class="locale" data-locale="ru">Числа, UTF-8, opt_table, QPS, RT и безопасные ошибки. <a href="examples.ru.html">Основные результаты</a> · <a href="../examples/README.md">Команды и исходники</a> · <a href="../examples/performance.html">Интерактивные графики и кардиограммы</a></p><p class="locale" data-locale="en" hidden>Numbers, UTF-8, opt_table, QPS, RT and checked errors. <a href="examples.en.html">Key results</a> · <a href="../examples/README.md">Commands and sources</a> · <a href="../examples/performance.html?lang=en">Interactive charts and waveforms</a></p></div>'
modes_notice='<div class="panel"><p class="locale" data-locale="ru"><strong>predict_seq → длина → индекс · opt_table → тег → читатель</strong><br>Прежний смешанный формат сохранён как [predict_seq,opt_table]. Отдельный [opt_table] теперь совместим с default. <a href="optional-modes.ru.html">Контракт, ограничения и переход</a></p><p class="locale" data-locale="en" hidden><strong>predict_seq → length → index · opt_table → tag → reader</strong><br>The former mixed format is retained by [predict_seq,opt_table]. Standalone [opt_table] now matches default. <a href="optional-modes.en.html">Contract, limits and migration</a></p></div>'
body=body.replace('<section id="full-guide">',modes_notice+'<section id="full-guide">',1)
body=body.replace('<section id="full-guide">',examples_notice+'<section id="full-guide">',1)
body=body.replace('<section id="full-guide">',notice+'<section id="full-guide">',1)
body=body.replace('<section id="full-guide">','<section id="full-guide" class="reading-intro"><h3 data-t="guideRoute"></h3><p data-t="guideRouteText"></p>')
modern_notice='<section class="panel"><h2>QSL · 06.11.2026</h2><p class="locale" data-locale="ru"><strong>Строгий STACK_LIMIT · однопроходное checked-чтение · native COPY/LIFETIME</strong><br>QPS — только самостоятельные числа; строки и длины не квантуются. <a href="modernization-2026-11-06/MODERNIZATION.ru.html">Новый контракт и миграция</a> · <a href="../qsl-bench/modernization/report.html">4 свежих Criterion-случая и кардиограммы</a>. Старые таблицы ниже сохраняют свои снимки.</p><p class="locale" data-locale="en" hidden><strong>Strict STACK_LIMIT · single-pass checked reads · native COPY/LIFETIME</strong><br>QPS packs scalar numbers only; strings and lengths are not quantized. <a href="modernization-2026-11-06/MODERNIZATION.en.html">Current contract and migration</a> · <a href="../qsl-bench/modernization/report.html">4 fresh Criterion cases and sample traces</a>. Historical tables below retain their snapshots.</p></section>'
body=body.replace('<section id="overview"',modern_notice+'<section id="overview"',1)

# Initial explanatory copy is visible even before script execution.
body=re.sub(r'(<([a-z0-9]+)\b[^>]*\bdata-t="([^"]+)"[^>]*>)(</\2>)',lambda m:m[1]+t(m[3])+m[4],body)

def embed(id,value):return '<script type="application/json" id="'+id+'">'+json.dumps(value,ensure_ascii=False,separators=(',',':')).replace('<','\\u003c')+'</script>'
dashboard=json.loads((docs/'data/features.json').read_text());dashboard['rows']+=json.loads((docs/'data/history-comparable.json').read_text())['rows']
page=head+body+embed('feature-data',dashboard)+embed('api-data',json.loads((docs/'data/api.json').read_text()))+embed('words',words)+embed('group-notes',json.loads((docs/'data/full-group-notes.json').read_text()))+'<script src="assets/full.js"></script><script src="assets/explorers.js"></script></body></html>'
(docs/'index.html').write_text(page,encoding='utf-8',newline='\n')
css=(docs/'assets/full.css').read_text();js=(docs/'assets/full.js').read_text();(docs/'QSL_standalone_RU_EN.html').write_text(page.replace('<link rel="stylesheet" href="assets/full.css">','<style>'+css+'</style>').replace('<script src="assets/full.js"></script>','<script>'+js+'</script>').replace('<link rel="stylesheet" href="assets/explorers.css">','<style>'+(docs/'assets/explorers.css').read_text()+'</style>').replace('<script src="assets/explorers.js"></script>','<script>'+(docs/'assets/explorers.js').read_text()+'</script>'),encoding='utf-8',newline='\n')
for lang in ['ru','en']:
 for name in ['guide','features','architecture','API','performance','opt-table-control','mathematics','safety','schema-hashes','examples','optional-modes']:
  p=docs/f'{name}.{lang}.md'
  if not p.exists():continue
  content,_=render(p.read_text(encoding='utf-8'),name+'-'+lang);content=content.replace('<h2','<h1',1).replace('</h2>','</h1>',1);H=head.replace('lang="ru"',f'lang="{lang}"').replace('<title>QSL · '+t('project')+'</title>',f'<title>QSL · {name} · {lang.upper()}</title>');(docs/f'{name}.{lang}.html').write_text(H+'<main class="book" style="max-width:1120px;padding:25px;margin:auto"><a href="index.html">QSL</a>'+content+'</main></body></html>',encoding='utf-8',newline='\n')
print('Full QSL HTML built with measured samples, region explorer, length graph, guide and API')

# Keep the nested current implementation contract reproducible with the site.
for lang in ["ru", "en"]:
 p=docs/"modernization-2026-11-06"/f"MODERNIZATION.{lang}.md"
 content,_=render(p.read_text(encoding="utf-8"),"modern-"+lang)
 content=content.replace("<h2","<h1",1).replace("</h2>","</h1>",1)
 title="QSL · Modernization · "+lang.upper()
 H=head.replace('lang="ru"',f'lang="{lang}"').replace(
  "<title>QSL · "+t("project")+"</title>","<title>"+title+"</title>"
 ).replace('href="assets/','href="../assets/')
 p.with_suffix(".html").write_text(
  H+'<main class="book" style="max-width:1120px;padding:25px;margin:auto">'
  +'<a href="../index.html">QSL</a>'+content+"</main></body></html>",
  encoding="utf-8",newline="\n")
