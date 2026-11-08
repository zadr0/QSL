from pathlib import Path
import hashlib,html,json,re

def complete_home(page,lang,repo,render):
    docs=repo/'docs';en=lang=='en';L=lambda ru,english:english if en else ru
    words=json.loads((docs/'data/full-words.json').read_text(encoding='utf-8'))
    words['scopeBoundary']=['QSL задаёт представление данных. Транспорт, TLS, версии и IPC выбирает приложение. Переносимый runtime находится в qsl-core.','QSL defines data representation. The application owns transport, TLS, version negotiation and IPC. Portable runtime code lives in qsl-core.']
    words['sourceNote']=['Пути API относятся к текущей ветке main и зафиксированному снимку исходников.','API paths refer to current main and the recorded source snapshot.']
    fragment=(docs/'assets/workshop.html').read_text(encoding='utf-8')
    def translate(m):
        pair=words.get(m['key'],[m['key'],m['key']]);return '<'+m['tag']+m['attrs']+'>'+html.escape(pair[1 if en else 0])+'</'+m['tag']+'>'
    fragment=re.sub(r'<(?P<tag>[a-z][a-z0-9]*)(?P<attrs>[^>]*\bdata-t="(?P<key>[^"]+)"[^>]*)>[^<]*</(?P=tag)>',translate,fragment)
    def select_locale(m):
        if m['locale']!=lang:return ''
        return '<'+m['tag']+re.sub(r'\s+hidden(?=\s|$)','',m['attrs'])+'>'+m['body']+'</'+m['tag']+'>'
    fragment=re.sub(r'<(?P<tag>p|h2)(?P<attrs>[^>]*data-locale="(?P<locale>ru|en)"[^>]*)>(?P<body>.*?)</(?P=tag)>',select_locale,fragment,flags=re.S)
    # Empty inputs remain native controls; only prose is translated at build time.
    fragment=re.sub(r'data-aria="([^"]+)" aria-label="[^"]*"',lambda m:'data-aria="'+m[1]+'" aria-label="'+html.escape(words[m[1]][1 if en else 0],quote=True)+'"',fragment)
    page=page.replace('<section class="chapter" id="strings">',fragment+'\n<section class="chapter" id="strings">',1)
    content,toc=render((docs/f'guide.{lang}.md').read_text(encoding='utf-8'),'full-guide-'+lang)
    attributes=''.join('<a href="attributes/'+lang+'/'+p.stem+'.html"><code>'+html.escape(p.stem)+'</code></a>' for p in sorted((docs/'attributes'/lang).glob('*.md')) if p.stem!='index')
    catalogue='<section id="attribute-catalogue" class="chapter"><header><h2>'+L('Все атрибуты','All attributes')+'</h2></header><p>'+L('Синтаксис, ограничения и совместимость каждой опции описаны отдельно. Атрибуты опциональны; порядок required → optional обязателен.','Each option has a separate syntax, constraints and compatibility reference. Attributes are optional; required → optional field order is mandatory.')+'</p><div class="attribute-index">'+attributes+'</div></section>'
    guide='<section id="full-guide" class="chapter"><header><h2>'+L('Полное руководство QSL','Complete QSL manual')+'</h2><a href="guide.'+lang+'.html">'+L('Отдельная страница / печать','Separate page / print')+'</a></header><p class="notice">'+L('Весь текст открыт сразу. Оглавление слева ведёт к нужной главе; API, атрибуты и примеры дополняют руководство. Команды установки пакетов относятся к выпущенному 0.2.0, изменения qsl-core — к текущей ветке main.','All text is visible immediately. Use the left contents to reach a chapter; API, attributes and examples supplement the manual. Package installation targets released 0.2.0; qsl-core changes belong to current main.')+'</p><article class="book full-guide">'+content+'</article></section>'
    page=page.replace('<section class="chapter" id="api">',catalogue+guide+'\n<section class="chapter" id="api">',1)
    links=[('workshop',L('Как устроен QSL','How QSL works')),('layout-lab',L('Битовая раскладка','Bit layout')),('qps-demo',L('Операнды QPS','QPS operands')),('event-demo',L('Индекс длины PS','PS length index')),('full-guide',L('Полное руководство','Complete manual')),('attribute-catalogue',L('Все атрибуты','All attributes'))]
    nav=''.join('<a href="#'+target+'">'+label+'</a>'for target,label in links)
    nav+='<h2>'+L('Главы руководства','Manual chapters')+'</h2><div class="doc-nav">'+''.join('<a href="#'+anchor+'">'+html.escape(label)+'</a>'for anchor,label in toc)+'</div>'
    page=page.replace('</aside>',nav+'</aside>',1)
    snapshot='<p class="notice">'+L('Прежние интерактивные графики всех возможностей сохранены как отдельный исторический снимок. Они не описывают скорость и строковые API текущей ветки.','The previous interactive feature charts remain available as a historical snapshot. They do not describe current-branch speed or string APIs.')+' <a href="archive/2026-11-07/index.html">'+L('Открыть снимок','Open snapshot')+'</a></p>'
    page=page.replace('<section class="chapter" id="reading">',snapshot+'<section class="chapter" id="reading">',1)
    page=page.replace('class="pipeline"><a href="https://github.com/zadr0/QSL/tree/main/qsl-core"','class="runtime-pipeline"><a href="https://github.com/zadr0/QSL/tree/main/qsl-core"',1)
    css=hashlib.sha256((docs/'assets/explorers.css').read_bytes()).hexdigest()[:10]
    page=page.replace('</head>','<link rel="stylesheet" href="assets/explorers.css?v='+css+'"></head>',1)
    version=hashlib.sha256((docs/'assets/workshop.js').read_bytes()).hexdigest()[:10]
    data=json.dumps(words,ensure_ascii=False,separators=(',',':')).replace('<','\\u003c')
    page=page.replace('</body>','<script id="workshop-words" type="application/json">'+data+'</script><script src="assets/workshop.js?v='+version+'" defer></script></body>',1)
    return page
