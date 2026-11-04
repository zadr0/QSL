from pathlib import Path
import json,sys,re,hashlib,zipfile,csv,datetime,statistics
root=Path(__file__).resolve().parents[1];out=root/'report';out.mkdir(exist_ok=True)
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
rows=[];profiles=[];rejections=[]
for m in sorted((root/'results').rglob('metrics.json')):
 meta=read(m)
 if 'case'not in meta:continue
 case=meta['case'];directory=m.parent;estimate=next((p for p in (directory/'criterion').rglob('estimates.json')if p.parent.name=='new'),None)
 if not estimate:continue
 before=directory.parent/'sources-before.json';after=directory.parent/'sources-after.json';source_ok=before.exists()and after.exists()and read(before)==read(after)
 parts=case.split('/');control=parts[0]=='control';batch=int(re.search(r'/b(\d+)',case)[1]);kind=parts[1]if not control else 'mixed'if 'consume'in parts else 'loop';operation=parts[2]if not control else parts[1];codec=parts[-1]if not control else '/'.join(parts[1:-1]);cm=read(directory/'case.json')if(directory/'case.json').exists()else{}
 est=read(estimate)['mean'];sample=read(estimate.parent/'sample.json');times=[t/n for t,n in zip(sample['times'],sample['iters'])]
 quality=bool(meta.get('metricsRevision',1)>=2 and meta.get('noiseAccepted')and source_ok and not meta.get('sampler')and meta['exitCode']==0 and meta.get('idleMaxPercent',100)<=5 and meta.get('idleSamples',0)>=3 and meta.get('warmupSeconds')==15 and meta.get('measurementSeconds')==10 and meta.get('samples')==200 and meta.get('confidence')==.99 and len(times)==200 and not any('collector' in x for x in directory.parts))
 useful=cm.get('useful_per_record');wire=cm.get('wire_total_for_fixture_ring',0)/cm.get('fixture_count',1)if cm else None
 r={'case':case,'layer':parts[0],'kind':kind,'operation':operation,'codec':codec,'batch':batch,'mean_batch_ns':est['point_estimate'],'low_batch_ns':est['confidence_interval']['lower_bound'],'high_batch_ns':est['confidence_interval']['upper_bound'],'time_ns':est['point_estimate']/batch,'low_ns':est['confidence_interval']['lower_bound']/batch,'high_ns':est['confidence_interval']['upper_bound']/batch,'useful_bytes':useful,'wire_bytes':wire,'p':useful*batch/est['point_estimate']if useful else None,'samples_batch_ns':times,'sample_cv':statistics.stdev(times)/statistics.mean(times)if len(times)>1 else 0,'eligible':quality,'source_ok':source_ok,'metrics':meta,'profile':directory.parent.name,'raw_directory':directory.relative_to(root).as_posix(),'wire_scope':'codec payload; HTTP2/TCP overhead excluded'if parts[0]=='transport'else'encoded record','instrumented':bool(meta.get('sampler'))}
 rows.append(r)
for p in (root/'results').rglob('progress.json'):
 data=read(p);data=data if isinstance(data,list)else[data]
 for r in data:
  if r.get('status')=='idle_gate_failed':rejections.append({'profile':p.parent.name,'case':r['case'],'reason':'idle gate failed; not measured'})
# Repetitions remain separate. Prefer the latest qualified row per case for the main display.
binary=root/'target/release/deps'
executables=list(binary.glob('comparison-*.exe')) if binary.exists()else []
current_binary=hashlib.sha256(max(executables,key=lambda p:p.stat().st_mtime).read_bytes()).hexdigest().upper()if executables else next((r['metrics']['binarySha256']for r in reversed(rows)),None)
selected={}
for r in rows:
 if r['metrics'].get('binarySha256')!=current_binary:continue
 old=selected.get(r['case'])
 if old is None or r['eligible']or not old['eligible']:selected[r['case']]=r
allocation=next((read(p)for p in sorted((root/'results').rglob('data.json'))if p.parent.name.startswith('allocation')and p.stat().st_size>10),[])
latency=next((read(p)for p in sorted((root/'results').rglob('data.json'))if p.parent.name.startswith('latency')and p.stat().st_size>10),None)
for r in rows:
 r['alloc']=next((a for a in allocation if a['kind']==r['kind']and a['codec']==r['codec']and a['operation']==r['operation']and a['batch']==r['batch']),None)
data={'status':'complete'if len([r for r in selected.values()if r['eligible']])==100 else'waiting_or_partial','binarySha256':current_binary,'generated_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'expected':100,'accepted':len([r for r in selected.values()if r['eligible']]),'rows':list(selected.values()),'observations':rows,'rejections':rejections,'allocations':allocation,'latency':latency,'environment':read(root/'environment.json'),'cache':read(root/'cache-cpuid.json'),'versions':read(root/'versions.json'),'matrix':read(root/'matrix.json')}
(out/'data.json').write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
fields=['case','profile','eligible','batch','mean_batch_ns','time_ns','low_ns','high_ns','useful_bytes','wire_bytes','p','sample_cv']
with(out/'measurements.csv').open('w',encoding='utf-8',newline='')as f:
 w=csv.DictWriter(f,fieldnames=fields,extrasaction='ignore');w.writeheader();w.writerows(rows)
with zipfile.ZipFile(out/'raw.zip','w',zipfile.ZIP_DEFLATED)as z:
 for p in (root/'results').rglob('*'):
  if p.is_file()and p.suffix in ['.json','.log','.zip']:z.write(p,p.relative_to(root).as_posix())
 # The compiler and executable digest evidence is independent of the source tree.
 for name in ['environment.json','cache-cpuid.json','pmc-sources.txt','versions.json','TOOLS.json','matrix.json']:z.write(root/name,name)
# Exact sources for every completed run that still matches current files; already archived old sources are retained in raw.zip.
project=root.parents[1]
for before in (root/'results').rglob('sources-before.json'):
 source=read(before)
 if not all((root/rel).is_file()and hashlib.sha256((root/rel).read_bytes()).hexdigest().upper()==h for rel,h in source.items()):continue
 path=out/(before.parent.name+'-sources.zip')
 with zipfile.ZipFile(path,'w',zipfile.ZIP_DEFLATED)as z:
  for rel in source:
   p=(root/rel).resolve();assert p.is_relative_to(project);z.write(p,p.relative_to(project).as_posix())
html=(root/'tools/report-template.html').read_text(encoding='utf-8').replace('/*DATA*/',json.dumps(data,ensure_ascii=False,separators=(',',':')).replace('<',chr(92)+'u003c'))
(root/'report.html').write_text(html,encoding='utf-8',newline='\n')
text='# Сравнение протоколов / Protocol comparison\n\n'
text+='Принято строгих случаев / qualified cases: '+str(data['accepted'])+' / 100.\n\n'
text+='Не прошедшие качество данные сохраняются отдельно и не используются для вывода о быстродействии. Rejected observations are retained and excluded from performance conclusions.\n\n'
text+='[Интерактивные графики / charts](report.html) · [Методика / method](METHOD.md) · [Кандидаты оптимизации](OPTIMIZATION_CANDIDATES.ru.md) · [Безопасность](SECURITY.ru.md)\n\n'
text+='| Case | T / record, ns | 99% CI | P, B/ns | Peak commit, MiB |\n|---|---:|---:|---:|---:|\n'
for r in data['rows']:
 if not r['eligible']or r['layer']=='control':continue
 text+=f"| {r['case']} | {r['time_ns']:.3f} | {r['low_ns']:.3f}–{r['high_ns']:.3f} | {r['p']:.4f} | {r['metrics']['peakJobBytes']/1048576:.2f} |\n"
(root/'RESULTS.md').write_text(text,encoding='utf-8',newline='\n')
try:
 import matplotlib
 matplotlib.use('Agg')
 import matplotlib.pyplot as plt
 fig,axes=plt.subplots(2,2,figsize=(13,8),layout='constrained')
 choices=[('numeric','write',1),('numeric','read',1),('mixed','write',16),('mixed','read',16)]
 for ax,(kind,op,batch)in zip(axes.flat,choices):
  chosen=[r for r in data['rows']if r['eligible']and r['layer']=='codec'and r['kind']==kind and r['operation']==op and r['batch']==batch]
  ax.set_title(f'{kind} / {op} / batch {batch}',loc='left')
  if not chosen:ax.text(.5,.5,'No qualified observations',ha='center',va='center',transform=ax.transAxes);ax.set_axis_off();continue
  labels=[r['codec']for r in chosen];values=[r['time_ns']for r in chosen];y=list(range(len(chosen)));ax.barh(y,values,color=['#135b6c'if'optimized'in r['codec']else'#68848b'for r in chosen]);ax.errorbar(values,y,xerr=[[r['time_ns']-r['low_ns']for r in chosen],[r['high_ns']-r['time_ns']for r in chosen]],fmt='none',color='#182c37',capsize=3);ax.set_yticks(y,labels);ax.invert_yaxis();ax.set_xlabel('ns / record, mean and marginal 99% CI')
 fig.suptitle('QSL · native prepared profiles · i5-7400 · accepted observations only',fontsize=14)
 fig.savefig(out/'overview.png',dpi=160);fig.savefig(out/'overview.svg');plt.close(fig)
except ImportError:pass
files=[p for p in out.rglob('*')if p.is_file()and p.name!='SHA256SUMS.txt'and not p.name.startswith('preview-')]
(out/'SHA256SUMS.txt').write_text(''.join(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+p.relative_to(out).as_posix()+'\n'for p in sorted(files)),encoding='utf-8',newline='\n')
print(f"Report: {data['accepted']}/100 qualified, {len(rows)} observations, {len(rejections)} idle rejections")
