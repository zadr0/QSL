from pathlib import Path
from html.parser import HTMLParser
import json,re,html
root=Path(__file__).resolve().parents[1]
class Page(HTMLParser):
 def __init__(self):super().__init__();self.ids=[];self.scripts={};self.current=None
 def handle_starttag(self,tag,attrs):
  a=dict(attrs)
  if 'id' in a:self.ids.append(a['id'])
  if tag=='script' and a.get('type')=='application/json':self.current=a.get('id');self.scripts[self.current]=''
 def handle_endtag(self,tag):
  if tag=='script':self.current=None
 def handle_data(self,data):
  if self.current:self.scripts[self.current]+=data
paths=[*root.glob('README*.md'),root/'SOURCE_AUDIT.md',*root.glob('docs/*.md'),*root.glob('docs/*.html'),*root.glob('docs/attributes/*/*.md'),*root.glob('docs/en/*.md'),*root.glob('docs/ru/*.md'),*root.glob('docs/releases/*'),root/'qsl-interpreter/tests/optional-v2/README.md']
checked=0;errors=[]
for p in paths:
 text=p.read_text(encoding='utf-8')
 if p.suffix=='.html':
  page=Page();page.feed(text)
  if len(page.ids)!=len(set(page.ids)):errors.append([str(p.relative_to(root)),'duplicate ids'])
  for value in page.scripts.values():json.loads(value)
  refs=re.findall(r'(?:href|src)="([^"]+)"',text)
 else:refs=re.findall(r'\]\(([^)]+)\)',text)
 for ref in refs:
  if ref.startswith(('http:','https:','mailto:','#','data:')):continue
  ref=html.unescape(ref).split('#')[0].split('?')[0]
  if not ref:continue
  checked+=1
  if not (p.parent/ref).exists():errors.append([str(p.relative_to(root)),ref])
api=json.loads((root/'docs/data/api.json').read_text(encoding='utf-8'))
assert any(x['kind']=='macro' and 'qsl_primitive' in x['signature'] for x in api)
assert any('QslPrimitive' in x['signature'] for x in api)
for row in api:
 source=root/row['path'];assert source.is_file(),row
 assert 0<row['line']<=len(source.read_text(encoding='utf-8').splitlines()),row
standalone=(root/'docs/QSL_standalone_RU_EN.html').read_text(encoding='utf-8')
assert 'src="assets/' not in standalone and 'href="assets/' not in standalone
assert 'QslBitmapDoc' in standalone
report={'current_files':len(paths),'checked_links':checked,'api_declarations':len(api),'broken':errors,'historical_archives':'preserved and excluded from current-link validation'}
print(json.dumps(report,ensure_ascii=False))
assert not errors
