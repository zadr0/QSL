from pathlib import Path
from html.parser import HTMLParser
import json,xml.etree.ElementTree as ET
root=Path(__file__).resolve().parents[1];docs=root/'docs';base='https://zadr0.github.io/QSL/'
class Head(HTMLParser):
 def __init__(self):super().__init__();self.meta=[];self.links=[];self.lang=None;self.json=[];self.reading=False
 def handle_starttag(self,tag,attrs):
  a=dict(attrs)
  if tag=='html':self.lang=a.get('lang')
  if tag=='meta':self.meta.append(a)
  if tag=='link':self.links.append(a)
  if tag=='script' and a.get('type')=='application/ld+json':self.reading=True;self.json.append('')
 def handle_endtag(self,tag):
  if tag=='script':self.reading=False
 def handle_data(self,text):
  if self.reading:self.json[-1]+=text
records=json.loads((docs/'data/seo-pages.json').read_text(encoding='utf-8'));by_url={r['url']:r for r in records if r['index']};titles=set()
for record in records:
 p=Head();p.feed((docs/record['path']).read_text(encoding='utf-8'))
 assert p.lang==record['lang']
 assert [x['href'] for x in p.links if x.get('rel')=='canonical']==[record['url']]
 assert len([x for x in p.meta if x.get('name')=='description'])==1
 assert json.loads(p.json[0])['@graph'][0]['codeRepository']=='https://github.com/zadr0/QSL'
 alts={x['hreflang']:x['href'] for x in p.links if x.get('rel')=='alternate' and 'hreflang'in x}
 assert alts==record['alternates']
 if record['index']:
  assert record['title'] not in titles,record['title'];titles.add(record['title'])
  for language,url in alts.items():
   assert url in by_url,(record['path'],url)
   assert record['url'] in by_url[url]['alternates'].values(),(record['path'],url,'nonreciprocal')
 else:assert any(x.get('name')=='robots' and x.get('content')=='noindex,follow' for x in p.meta)
namespace={'s':'http://www.sitemaps.org/schemas/sitemap/0.9'};tree=ET.parse(docs/'sitemap.xml');urls=[x.text for x in tree.findall('s:url/s:loc',namespace)]
assert set(urls)==set(by_url) and len(urls)==len(set(urls))
text=(docs/'en.html').read_text(encoding='utf-8');assert '<html lang="en">'in text
assert 'data-locale="en" hidden' not in text
assert 'QSL: exchange of data described in advance' in text
assert not (docs/'robots.txt').exists(),'Project-local robots.txt must not masquerade as host-root policy'
import struct
png=(docs/'assets/social-card.png').read_bytes()
assert png[:8]==b'\x89PNG\r\n\x1a\n'
assert struct.unpack('>II',png[16:24])==(1200,630)
print(json.dumps({'seo_pages':len(records),'canonical_sitemap_urls':len(urls),'reciprocal_languages':True,'ssr_english_home':True,'schema_json_ld':True,'preview_dimensions':[1200,630],'host_root_robots_scope_documented':True}))
