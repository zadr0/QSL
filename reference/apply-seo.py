from pathlib import Path
import html,json,re,os,secrets,urllib.parse,xml.etree.ElementTree as ET
root=Path(__file__).resolve().parents[1]
docs=root/'docs';base='https://zadr0.github.io/QSL/';tag='v0.2.0'
# A real English HTML entry point is available before JavaScript executes.
home=(docs/'index.html').read_text(encoding='utf-8');words=json.loads((docs/'data/full-words.json').read_text(encoding='utf-8'))
def localized_open(m):
 attrs=re.sub(r'\s+hidden(?:="[^"]*")?','',m[2])
 return '<'+m[1]+attrs+(' hidden' if m[3]!='en' else '')+'>'
en=re.sub(r'<([a-z][\w]*)([^>]*\bdata-locale="(ru|en)"[^>]*)>',localized_open,home)
en=en.replace('<html lang="ru">','<html lang="en">',1)
def translate(m):return m[1]+html.escape(words[m[3]][1])+m[5] if m[3] in words else m[0]
en=re.sub(r'(<([a-z][\w]*)\b[^>]*\bdata-t="([^"]+)"[^>]*>)(.*?)(</\2>)',translate,en,flags=re.S)
(docs/'en.html').write_text(en,encoding='utf-8')
# Share images are deterministic typography, with no timing or ranking claims.
card=docs/'assets/social-card.png'
if not card.exists():
 from PIL import Image,ImageDraw,ImageFont
 image=Image.new('RGB',(1200,630),'#f4f5f7');draw=ImageDraw.Draw(image)
 draw.rounded_rectangle((38,38,1162,592),24,fill='#18232e')
 font=lambda size:ImageFont.truetype('C:/Windows/Fonts/times.ttf',size)
 draw.text((88,66),'QSL',font=font(126),fill='#f4f5f7')
 draw.text((956,116),'0.2.0',font=font(42),fill='#8ecbff')
 draw.text((94,244),'Binary serialization for Rust',font=font(55),fill='#ffffff')
 draw.text((94,318),'Explicit schemas. Generated codecs. Prepared buffers.',font=font(29),fill='#b8cad9')
 for x,label in [(94,'qsl-lexer'),(468,'qsl-interpreter'),(842,'runtime')]:
  draw.rounded_rectangle((x,415,x+266,484),12,outline='#8ecbff',width=2)
  draw.text((x+22,429),label,font=font(30),fill='#e6edf5')
 draw.text((384,431),'→',font=font(31),fill='#86d9bf');draw.text((758,431),'→',font=font(31),fill='#86d9bf')
 draw.text((94,536),'github.com/zadr0/QSL',font=font(26),fill='#b8cad9')
 image.save(card,optimize=True)
(docs/'assets/favicon.svg').write_text('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect width="64" height="64" rx="12" fill="#18232e"/><text x="32" y="47" text-anchor="middle" font-family="Times New Roman,serif" font-size="49" fill="#8ecbff">Q</text></svg>\n',encoding='utf-8')
# Old reports remain useful, but their original source and data are not rewritten.
reports={root/'examples/performance.html':docs/'reports/examples.html',root/'qsl-bench/modernization/report.html':docs/'reports/modernization.html'}
origins={}
for source,target in reports.items():
 target.parent.mkdir(exist_ok=True);text=source.read_text(encoding='utf-8')
 if '</head>' not in text:
  text=re.sub(r'(<html\b[^>]*>)',r'\1<head>',text,count=1)
  text=text.replace('</style>','</style></head><body>',1)
  if '</html>' in text:text=text.replace('</html>','</body></html>',1)
  else:text+='</body></html>'
 if source.name=='performance.html':text=text.replace('performance/','https://raw.githubusercontent.com/zadr0/QSL/'+tag+'/examples/performance/')
 target.write_text(text,encoding='utf-8');origins[target.resolve()]=source.resolve()
topics={
 'guide':(('Руководство QSL — схемы и бинарная сериализация Rust','QSL guide — schemas and Rust binary serialization'),('Полное руководство QSL: формат схем, строки UTF-8, optional-поля, память, проверки и генерация Rust-кода.','The complete QSL guide: schema format, UTF-8 strings, optional fields, memory, validation and specialized Rust code generation.')),
 'architecture':(('Архитектура QSL — lexer, interpreter и runtime Rust','QSL architecture — lexer, interpreter and Rust runtime'),('Архитектура компилятора QSL: общий сборщик схем, независимые backend, статические таблицы и переносимый runtime.','QSL compiler architecture: one schema builder, independent backends, static dispatch tables and a portable Rust runtime.')),
 'API':(('Rust API QSL — кодеки, схемы и буферы','QSL Rust API — codecs, schemas and buffers'),('Индекс API QSL 0.2.0: сборщик схем, методы сериализации, проверка входа, буферы и ссылки на исходники по тегу.','QSL 0.2.0 API reference: schema builders, serialization methods, validation, buffers and links to tagged Rust sources.')),
 'optional-modes':(('Опциональные поля QSL — bitmap, predict_seq и opt_table','QSL optional fields — bitmap, predict_seq and opt_table'),('Различия BM, PS и OT в QSL: маски 8–128 бит, индекс по длине, функции чтения, формат кадра и ограничения.','BM, PS and OT in QSL: 8–128-bit masks, length-derived indices, reader dispatch, frame format and compatibility constraints.')),
 'safety':(('Безопасность QSL — проверка входа и границ данных','QSL safety — checked decoding and data bounds'),('Проверки QSL: длина кадра, диапазоны, UTF-8, распаковка, MAX_DATA_SIZE и отдельный контракт unchecked-методов.','QSL input validation: frame length, ranges, UTF-8, decompression limits, MAX_DATA_SIZE and explicit unchecked API contracts.')),
 'schema-hashes':(('Хэши схем QSL — строгая и лёгкая идентичность','QSL schema hashes — strict and light identities'),('Детерминированные хэши QSL по схеме и формату: xxHash, xxHash128, порядок полей, BM v2 и ограничения совместимости.','Deterministic QSL schema identities: xxHash, xxHash128, field order, BM v2 and the limits of hash-based compatibility checks.')),
 'custom-types':(('Примитивы и вложенные структуры QSL для Rust','QSL custom primitives and nested Rust structures'),('QslCustom, проверяемые макросы примитивов, регистрация типов, native-регионы, COPY и LIFETIME в QSL 0.2.0.','QslCustom, validating scalar-wrapper macros, type registration, native regions, COPY and LIFETIME in QSL 0.2.0.')),
 'migration-0.2.0':(('Переход на QSL 0.2.0 — BM v2 и API Rust','Migrating to QSL 0.2.0 — BM v2 and Rust APIs'),('Изменения QSL 0.2.0: новый BM-формат, общие читатели OT, вложенные типы, пользовательские примитивы и удаление Windows-модулей.','QSL 0.2.0 migration: new bitmap format, shared OT readers, nested types, custom primitives and removal of Windows-specific modules.')),
 'quickstart':(('QSL для Rust — установка и первый пример','QSL for Rust — installation and quick start'),('Установите QSL 0.2.0, опишите схему и соберите первый Rust-кодек с проверяемой сериализацией и восстановлением.','Install QSL 0.2.0, describe a schema and build your first Rust binary codec with checked serialization and deserialization.')),
 'features':(('Возможности QSL и сохранённые измерения','QSL features and recorded measurements'),('Возможности QSL, условия сравнения и исторические измерения Rust-кодеков. Результаты привязаны к версиям исходников.','QSL features, comparison conditions and historical Rust codec measurements, each tied to its captured source version.')),
 'performance':(('Производительность QSL — методика и ограничения замеров','QSL performance — methodology and measurement limits'),('Методика Criterion для QSL, полезные байты за нс, интервалы доверия и ограничения исторических результатов.','QSL Criterion methodology, useful bytes per nanosecond, confidence intervals and limits of recorded performance results.')),
 'opt-table-control':(('Сравнение default и opt_table в QSL','QSL default and opt_table comparison'),('Исторический контроль optional-полей QSL: размеры схем, прогретые буферы и условия сравнения default и opt_table.','Historical QSL optional-field control: schema sizes, prepared buffers and conditions for comparing default and opt_table.')),
 'mathematics':(('Математика QSL — ранги, операнды и границы BM','QSL mathematics — ranks, operands and bitmap bounds'),('Обратимая упаковка QSL, разрядности, индексы optional-полей, P=M/T и границы доказательств Lean4.','QSL lossless packing, bit widths, optional-field indices, P=M/T and the scope of Lean4 arithmetic proofs.')),
 'examples':(('Примеры QSL — схемы, UTF-8 и безопасное чтение','QSL examples — schemas, UTF-8 and checked decoding'),('Рабочие примеры QSL для Rust: числа, строки, optional-поля, QPS, RT, макросы и обработка повреждённых данных.','Runnable QSL Rust examples: numbers, strings, optional fields, QPS, RT, macros and rejection of malformed input.')),
 '0.2.0':(('QSL 0.2.0 — заметки выпуска и переход','QSL 0.2.0 — release notes and migration'),('Выпуск QSL 0.2.0: изменения формата и API, пакеты crates.io, проверки, исходники и документация RU/EN.','QSL 0.2.0 release: wire/API changes, crates.io packages, verification evidence, source tags and RU/EN documentation.')),
}
topics['SEO']=(('Поиск и индексация документации QSL','QSL documentation discovery and indexing'),('Техническое SEO документации QSL: canonical, RU/EN, sitemap, GitHub Pages, структурированные данные и границы индексации.','QSL documentation SEO: canonical URLs, RU/EN pages, sitemap, GitHub Pages, structured data and indexing boundaries.'))
def url(path):return base+path.relative_to(docs).as_posix()
def github_ref(path):
 relative=path.relative_to(root).as_posix()
 if path.suffix.lower() in ('.png','.svg','.jpg','.jpeg','.zip','.csv','.gz'):return 'https://raw.githubusercontent.com/zadr0/QSL/'+tag+'/'+relative
 return 'https://github.com/zadr0/QSL/'+('tree/' if path.is_dir() else 'blob/')+tag+'/'+relative

def rewrite_refs(text,path,origin):
 def fix(m):
  value=html.unescape(m[3])
  if value.startswith(('https:','http:','data:','mailto:','#','javascript:')) or '${' in value:return m[0]
  parsed=urllib.parse.urlsplit(value)
  if not parsed.path:return m[0]
  target=(origin.parent/urllib.parse.unquote(parsed.path)).resolve()
  if not target.is_relative_to(root):return m[0]
  if target in {x.resolve():y.resolve() for x,y in reports.items()}:target={x.resolve():y.resolve() for x,y in reports.items()}[target]
  if target.is_relative_to(docs.resolve()):new=os.path.relpath(target,path.parent).replace('\\','/')
  else:new=github_ref(target)
  if parsed.query:new+='?'+parsed.query
  if parsed.fragment:new+='#'+parsed.fragment
  return m[1]+'='+m[2]+html.escape(new,quote=True)+m[2]
 return re.sub(r'(href|src)=(\x22|\x27)(.*?)\2',fix,text)
records=[]
paths=[docs/'index.html',docs/'en.html',*docs.glob('*.ru.html'),*docs.glob('*.en.html'),*docs.glob('releases/*.html'),*docs.glob('modernization-2026-11-06/*.html'),*docs.glob('reports/*.html'),docs/'QSL_standalone_RU_EN.html']
for path in dict.fromkeys(paths):
 relative=path.relative_to(docs).as_posix();text=path.read_text(encoding='utf-8')
 lang='en' if path.name=='en.html' or '.en.html' in path.name else 'ru'
 li=0 if lang=='ru' else 1;stem=re.sub(r'\.(ru|en)\.html$','',path.name)
 indexed='reports/' not in relative and 'modernization-' not in relative and path.name!='QSL_standalone_RU_EN.html'
 canonical=base if path.name in ('index.html','QSL_standalone_RU_EN.html') else url(path)
 if path.name in ('index.html','en.html','QSL_standalone_RU_EN.html'):
  title=('QSL — бинарная сериализация и компилятор схем Rust','QSL — Rust binary serialization and schema compiler')[li]
  description=('QSL генерирует Rust-кодеки по явной схеме: типы, UTF-8, optional-поля, проверка входа и подготовленные буферы. Документация RU/EN.','QSL is a Rust binary serialization library and schema compiler for explicit data layouts, UTF-8, optional fields and prepared buffers.')[li]
  alternates={'ru':base,'en':base+'en.html','x-default':base}
 else:
  default=((f'QSL — {stem}',f'QSL — {stem}'),('Документация проекта QSL и Rust-кодеков.','QSL project and Rust codec documentation.'))
  labels,descriptions=topics.get(stem,default);title=labels[li];description=descriptions[li]
  other=path.with_name(path.name.replace('.ru.html','.en.html') if lang=='ru' else path.name.replace('.en.html','.ru.html'))
  alternates={lang:canonical}
  if other.exists() and other!=path:alternates['en' if lang=='ru' else 'ru']=url(other)
  alternates['x-default']=alternates.get('en',canonical)
 # Remove only managed metadata; a rebuild or second pass stays idempotent.
 text=re.sub(r'<title>.*?</title>','',text,count=1,flags=re.S)
 text=re.sub(r'<meta\b[^>]*(?:name|property)="(?:description|robots|og:[^"]+|twitter:[^"]+)"[^>]*>','',text)
 text=re.sub(r'<link\b[^>]*rel="(?:canonical|alternate|sitemap|icon)"[^>]*>','',text)
 text=re.sub(r'<script[^>]*id="seo-data"[^>]*>.*?</script>','',text,flags=re.S)
 image=base+'assets/social-card.png'
 tags=['<title>'+html.escape(title)+'</title>','<meta name="description" content="'+html.escape(description,quote=True)+'">','<meta name="robots" content="'+('index,follow,max-image-preview:large' if indexed else 'noindex,follow')+'">','<link rel="canonical" href="'+canonical+'">','<link rel="sitemap" type="application/xml" href="'+base+'sitemap.xml">','<link rel="icon" type="image/svg+xml" href="'+base+'assets/favicon.svg">']
 for language,href in alternates.items():tags.append('<link rel="alternate" hreflang="'+language+'" href="'+href+'">')
 for prop,value in {'og:type':'website' if stem in ('index.html','en.html') else 'article','og:site_name':'QSL','og:title':title,'og:description':description,'og:url':canonical,'og:locale':'ru_RU' if lang=='ru' else 'en_US','og:image':image,'og:image:width':'1200','og:image:height':'630','og:image:alt':'QSL 0.2.0 — binary serialization for Rust'}.items():tags.append('<meta property="'+prop+'" content="'+html.escape(value,quote=True)+'">')
 for name,value in {'twitter:card':'summary_large_image','twitter:title':title,'twitter:description':description,'twitter:image':image}.items():tags.append('<meta name="'+name+'" content="'+html.escape(value,quote=True)+'">')
 graph={'@context':'https://schema.org','@graph':[{'@type':'SoftwareSourceCode','@id':base+'#software','name':'QSL','version':'0.2.0','description':'Schema-driven binary serialization and specialized Rust code generation for explicit data layouts.','codeRepository':'https://github.com/zadr0/QSL','programmingLanguage':'Rust','runtimePlatform':'Rust 1.98+','url':base,'license':'https://spdx.org/licenses/MIT.html','isAccessibleForFree':True},{'@type':'WebPage','@id':canonical+'#webpage','url':canonical,'name':title,'description':description,'inLanguage':lang,'about':{'@id':base+'#software'}}]}
 tags.append('<script type="application/ld+json" id="seo-data">'+json.dumps(graph,ensure_ascii=False,separators=(',',':')).replace('<','\\u003c')+'</script>')
 text=text.replace('</head>','\n'+'\n'.join(tags)+'\n</head>',1)
 text=re.sub(r'<html lang="[^"]+">','<html lang="'+lang+'">',text,count=1)
 text=rewrite_refs(text,path,origins.get(path.resolve(),path.resolve()))
 if lang=='en':text=text.replace('href="index.html">QSL','href="en.html">QSL').replace('href="../index.html">QSL','href="../en.html">QSL')
 if 'id="qsl-language-links"' not in text:
  nav='<nav id="qsl-language-links" class="footer" aria-label="Documentation languages"><a href="'+base+'" hreflang="ru">Русский</a> · <a href="'+base+'en.html" hreflang="en">English</a> · <a href="'+base+'sitemap.xml">Sitemap</a></nav>'
  text=text.replace('</body>',nav+'</body>',1)
 path.write_text(text,encoding='utf-8')
 records.append({'path':relative,'url':canonical,'lang':lang,'title':title,'description':description,'index':indexed,'alternates':alternates})
ns='http://www.sitemaps.org/schemas/sitemap/0.9';xhtml='http://www.w3.org/1999/xhtml';ET.register_namespace('',ns);ET.register_namespace('xhtml',xhtml);tree=ET.Element('{'+ns+'}urlset')
for record in records:
 if not record['index']:continue
 item=ET.SubElement(tree,'{'+ns+'}url');ET.SubElement(item,'{'+ns+'}loc').text=record['url'];ET.SubElement(item,'{'+ns+'}lastmod').text='2026-11-08'
 for language,href in record['alternates'].items():ET.SubElement(item,'{'+xhtml+'}link',{'rel':'alternate','hreflang':language,'href':href})
ET.indent(tree);ET.ElementTree(tree).write(docs/'sitemap.xml',encoding='utf-8',xml_declaration=True)
(docs/'data/seo-pages.json').write_text(json.dumps(records,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
key_config=root/'reference/indexnow.json'
if not key_config.exists():
 key=secrets.token_hex(16);(docs/(key+'.txt')).write_text(key,encoding='utf-8');key_config.write_text(json.dumps({'host':'zadr0.github.io','keyLocation':base+key+'.txt','keyFile':key+'.txt'},indent=2)+'\n',encoding='utf-8')
(root/'reference/robots-root.example.txt').write_text('User-agent: *\nAllow: /QSL/\nSitemap: '+base+'sitemap.xml\n',encoding='utf-8')
print(json.dumps({'seo_pages':len(records),'sitemap_urls':sum(x['index'] for x in records),'english_home':'en.html','root_robots':'host-level; project-local robots.txt is deliberately not substituted for it','reports':'published copies; original historical artifacts preserved'}))
