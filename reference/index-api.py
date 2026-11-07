from pathlib import Path
import re,json,hashlib
root=Path(__file__).resolve().parents[1];repo=root;items=[]
def mask_rust(s):
 out=list(s);i=0
 def clear(a,b):
  for k in range(a,b):
   if s[k]!='\n':out[k]=' '
 while i<len(s):
  start=i
  if s.startswith('//',i):
   end=s.find('\n',i);end=len(s) if end<0 else end;clear(i,end);i=end;continue
  if s.startswith('/*',i):
   j=i+2;depth=1
   while j<len(s) and depth:
    if s.startswith('/*',j):depth+=1;j+=2
    elif s.startswith('*/',j):depth-=1;j+=2
    else:j+=1
   clear(i,j);i=j;continue
  raw=re.match(r'(?:b?r)(#*)"',s[i:])
  if raw:
   close='"'+raw[1];end=s.find(close,i+raw.end());end=len(s) if end<0 else end+len(close);clear(i,end);i=end;continue
  if s[i]=='"':
   j=i+1
   while j<len(s):
    if s[j]=='\\':j+=2
    elif s[j]=='"':j+=1;break
    else:j+=1
   clear(i,min(j,len(s)));i=j;continue
  char=re.match(r"'(?:\\(?:u\{[0-9a-fA-F]+\}|x[0-9a-fA-F]{2}|.)|[^'\n\\])'",s[i:])
  if char:clear(i,i+char.end());i+=char.end();continue
  i+=1
 return ''.join(out)
pattern=re.compile(r'^\s*pub\s+(?:(?:unsafe|const|async)\s+)*(fn|struct|enum|trait|type|const|static|mod|use)\b',re.M)
for folder in ['qsl-core/src','qsl/src','qsl/macros/src','qsl-lexer/src','qsl-interpreter/src']:
 for p in sorted((root/folder).rglob('*.rs')):
  text=p.read_text(encoding='utf-8');masked=mask_rust(text)
  for match in pattern.finditer(masked):
   start=match.start();start+=len(masked[start:match.end()])-len(masked[start:match.end()].lstrip());kind=match[1];tail=masked[match.end():];stop=re.search(r';' if kind=='use' else r'[;{]',tail)
   if not stop:continue
   end=match.end()+stop.end();signature=' '.join(text[start:end].split());line=text.count('\n',0,start)+1
   items.append({'module':p.relative_to(root/folder).with_suffix('').as_posix(),'path':p.relative_to(root).as_posix(),'line':line,'kind':kind,'signature':signature,'doc':''})
   if kind=='trait' and masked[end-1]=='{':
    depth=1;j=end
    while j<len(masked) and depth:
     depth+=(masked[j]=='{')-(masked[j]=='}');j+=1
    body=masked[end:j-1]
    for method in re.finditer(r'^\s*(?:(?:unsafe|async|const)\s+)*fn\s+',body,re.M):
     a=end+method.start();a+=len(masked[a:end+method.end()])-len(masked[a:end+method.end()].lstrip());stop=re.search(r'[;{]',masked[end+method.end():j-1])
     if stop:
      b=end+method.end()+stop.end();items.append({'module':p.relative_to(root/folder).with_suffix('').as_posix(),'path':p.relative_to(root).as_posix(),'line':text.count('\n',0,a)+1,'kind':'trait method','signature':' '.join(text[a:b].split()),'doc':''})
# Exported declarative macros are APIs too; generated Rust string literals stay masked.
for folder in ['qsl-core/src','qsl-lexer/src','qsl-interpreter/src','qsl/src']:
 for p in sorted((root/folder).rglob('*.rs')):
  text=p.read_text(encoding='utf-8');masked=mask_rust(text)
  for m in re.finditer(r'macro_rules!\s+(\w+)',masked):
   if '#[macro_export]' not in masked[max(0,m.start()-150):m.start()]:continue
   items.append({'module':p.relative_to(root/folder).with_suffix('').as_posix(),'path':p.relative_to(root).as_posix(),'line':text.count('\n',0,m.start())+1,'kind':'macro','signature':'macro_rules! '+m[1]+' { ... }','doc':''})
items.sort(key=lambda x:(x['path'],x['line']));(repo/'docs/data/api.json').write_text(json.dumps(items,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
for lang in ['en','ru']:
 ru=lang=='ru';head=('# Индекс публичных объявлений\n\n[English](API.en.md) · [Архитектура](architecture.ru.md)\n\n' if ru else '# Public declaration index\n\n[Русский](API.ru.md) · [Architecture](architecture.en.md)\n\n')
 head+=(f'Найдено {len(items)} явно публичных объявлений и сигнатур методов trait в текущих исходниках. Это source-index, а не отражение всех платформенных export-символов. Учитывайте feature rust-macros/diagnostics; native/Windows зависимость удалена; кодогенераторские строковые литералы не считаются API. Пути и строки относятся к локальному снимку из [manifest](../reference/core-refactor-2026-11-08/source-manifest.json). Индекс относится к текущей ветке main с qsl-core после опубликованного 0.2.0.\n\n' if ru else f'{len(items)} explicitly public declarations and trait method signatures from current sources. This is a source index, not reflection of every platform export. rust-macros/diagnostics feature gates still apply; no native/Windows dependency remains; generator string literals are excluded. Paths/lines refer to the local snapshot in the [manifest](../reference/core-refactor-2026-11-08/source-manifest.json). This index follows current main, including qsl-core extraction after published 0.2.0.\n\n')
 module=None
 for x in items:
  if x['path']!=module:module=x['path'];head+=f'## {module}\n\n'
  head+=f"`{x['path']}:{x['line']}`\n\n```rust\n{x['signature']}\n```\n\n"
 (repo/f'docs/API.{lang}.md').write_text(head.rstrip()+"\n",encoding='utf-8',newline='\n')
 (repo/f'reference/API.{lang}.md').write_text(('# Текущий API\n\n[Актуальный индекс](../docs/API.ru.md).\n' if ru else '# Current API\n\n[Current index](../docs/API.en.md).\n'),encoding='utf-8')
print('Indexed declarations:',len(items))
