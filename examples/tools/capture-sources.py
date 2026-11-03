from pathlib import Path
import datetime, hashlib, json, sys
root=Path(__file__).resolve().parents[2]
output=Path(sys.argv[1]); phase=sys.argv[2]
files=[root/'qsl/Cargo.toml',root/'qsl/Cargo.lock',*sorted((root/'qsl/src').rglob('*.rs')),*sorted((root/'qsl/macros').rglob('*.rs')),root/'qsl/macros/Cargo.toml']
files += [p for p in (root/'examples').rglob('*') if p.is_file() and p.suffix in ['.rs','.qsl','.deq','.toml','.lock'] and not any(x in p.parts for x in ['target','results'])]
data={'captured_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'phase':phase,'sources':{p.relative_to(root).as_posix():hashlib.sha256(p.read_bytes()).hexdigest().upper() for p in sorted(set(files))}}
output.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
