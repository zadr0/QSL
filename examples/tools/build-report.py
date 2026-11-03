"""Build a self-contained report from the exact completed Criterion run."""
from pathlib import Path
import csv, datetime, hashlib, io, json, math, re, shutil, sys, zipfile
try:
    import matplotlib
except ImportError:
    raise SystemExit("Install plotting dependencies: python -m pip install -r examples/tools/requirements-report.txt")
root = Path(__file__).resolve().parents[2]
bases = [Path(arg).resolve() for arg in sys.argv[1:]]
assert bases, "Pass one or more completed run directories"
examples = root / "examples"
dest = examples / "performance"
dest.mkdir(exist_ok=True)
load = lambda p: json.loads(p.read_text(encoding="utf-8"))
rows, profiles, executions = [], [], []
core_sources = None
for index, base in enumerate(bases):
    evidence = load(base / "memory.json")
    assert evidence["exitCode"] == 0
    before, after = load(base / "source-before.json"), load(base / "source-after.json")
    assert before["sources"] == after["sources"], "Sources changed during the measured run"
    core={k:v for k,v in before["sources"].items() if k.startswith("qsl/")}
    if core_sources is None: core_sources=core
    else: assert core_sources==core, "Different codec implementations cannot share this report"
    profile="main" if index==0 else f"control-{index}"
    prefix="" if index==0 else profile+"/"
    folder=dest/prefix;folder.mkdir(exist_ok=True)
    metadata={r["case"]:r for r in load(base/"cases.json")}
    measured=set()
    for estimates_path in sorted(base.glob("*/new/estimates.json")):
        raw_folder=estimates_path.parent
        bench=load(raw_folder/"benchmark.json");name=bench["full_id"]
        assert name in metadata, name
        e=load(estimates_path)["mean"];ci=e["confidence_interval"];sample=load(raw_folder/"sample.json")
        assert len(sample["iters"])==len(sample["times"])==200
        assert abs(ci["confidence_level"]-.99)<1e-8
        ns,lo,hi=e["point_estimate"],ci["lower_bound"],ci["upper_bound"]
        assert 0<lo<=ns<=hi and math.isfinite(hi)
        row=dict(metadata[name]);row.update(profile=profile,time_ns=ns,time_low=lo,time_high=hi,p=row["useful_bytes"]/ns,p_low=row["useful_bytes"]/hi,p_high=row["useful_bytes"]/lo,sample_ns=[t/i for t,i in zip(sample["times"],sample["iters"])],sample_iterations=sample["iters"],sample_times_ns=sample["times"],sampling_mode=sample["sampling_mode"])
        if "LZ_inline" in name:
            row["wire_mode"]="raw_fallback" if row["wire_min"]>=row["useful_bytes"] else "compressed"
        rows.append(row);measured.add(name)
    pattern=evidence.get("filter","")
    expected={name for name in metadata if not pattern or re.search(pattern,name)}
    assert measured==expected,"Incomplete or stale measurement directory"
    assert "affinity=0x4" in (base/"run.log").read_text(encoding="utf-8"),"Unexpected affinity"
    for name in ["source-before.json","source-after.json","memory.json","cases.json","run.log"]:
        shutil.copyfile(base/name,folder/name)
    with zipfile.ZipFile(folder/"criterion-raw.zip","w",zipfile.ZIP_DEFLATED) as z:
        for path in sorted(base.glob("*/new/*.json")):z.write(path,path.relative_to(base).as_posix())
    archive=folder/"measured-sources.zip"
    current=all((root/name).exists() and hashlib.sha256((root/name).read_bytes()).hexdigest().upper()==digest for name,digest in before["sources"].items())
    if current:
        with zipfile.ZipFile(archive,"w",zipfile.ZIP_DEFLATED) as z:
            for name in before["sources"]:z.write(root/name,name)
            for name in ["examples/tools/measure.ps1","examples/tools/capture-sources.py"]:z.write(root/name,name)
    else:
        assert archive.exists(),"Capture measured sources before changing them"
        with zipfile.ZipFile(archive) as z:
            assert all(hashlib.sha256(z.read(name)).hexdigest().upper()==digest for name,digest in before["sources"].items()),"Archived source hash mismatch"
    profiles.append({"id":profile,"cases":len(measured),"source_manifest":prefix+"source-before.json","source_archive":prefix+"measured-sources.zip","raw_archive":prefix+"criterion-raw.zip","execution":evidence})
    executions.append(evidence)
assert len({r["case"] for r in rows})==len(rows),"Never pool duplicate cases across binaries"
memory={"peakJobBytes":max(e["peakJobBytes"] for e in executions),"limitGiB":8,"exitCode":0,"runs":len(executions)}
hardware={"os":"Windows 11 Pro","os_version":"10.0.26200","cpu":"Intel Core i5-7400 @ 3.00 GHz","ram_gib":24,"ram_mhz":2133,"rustc":"1.98.0","llvm":"22.1.8","target":"x86_64-pc-windows-msvc","criterion":"0.8.2","affinity":"0x4","priority":"above-normal"}
report={"title":"QSL examples","created_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"hardware":hardware,"policy":{"warmup_seconds":15,"measurement_seconds":10,"samples":200,"confidence":.99,"memory_limit_gib":8,"fixtures":1024,"time_statistic":"Criterion arithmetic mean (ns/op)","index":"P=M/T; M is original useful bytes; T is ns/op","waveform":"One point is a Criterion batch mean, not individual latency or p99"},"execution":memory,"profiles":profiles,"rows":rows}
(dest/"measurements.json").write_text(json.dumps(report,ensure_ascii=False,indent=2)+"\n",encoding="utf-8",newline="\n")
fields=["case","group","operation","contract","profile","wire_mode","useful_bytes","wire_bytes","wire_min","wire_max","time_ns","time_low","time_high","p","p_low","p_high","fixtures","note"]
with (dest/"measurements.csv").open("w",encoding="utf-8",newline="") as stream:
    writer=csv.DictWriter(stream,fieldnames=fields,extrasaction="ignore");writer.writeheader();writer.writerows(rows)

groups={
 "numbers_8":("Числа: 8 u64(0..15)","Numbers: 8 u64(0..15)"),
 "strings_utf8_1024_uniform":("Строки: повторяющийся UTF-8, 2048 B","Strings: repeated UTF-8, 2048 B"),
 "strings_utf8_1024":("Строки: 1024 символа UTF-8","Strings: 1024 UTF-8 characters"),
 "whole_qps_ascii_1024":("Общий QPS: 1024 байта ASCII","Whole QPS: 1024 ASCII bytes"),
 "buffers":("Выход: новая Vec и статический RT","Output: fresh Vec and static RT"),
 "safety":("Валидация без создания объекта","Validation without constructing a value")}
for n in [1,2,3,5,10]:groups[f"optional_N{n}"]=(f"Optional: N={n}",f"Optional: N={n}")
order=["numbers_8","strings_utf8_1024","strings_utf8_1024_uniform",*[f"optional_N{n}" for n in [1,2,3,5,10]],"whole_qps_ascii_1024","buffers","safety"]
report["groups"]={g:{"ru":groups[g][0],"en":groups[g][1]} for g in order if any(r["group"]==g for r in rows)}
(dest/"measurements.json").write_text(json.dumps(report,ensure_ascii=False,indent=2)+"\n",encoding="utf-8",newline="\n")

def number(v):return f"{v:.3f}"
text="""# Производительность примеров / Example performance

[Примеры / Examples](README.md) · [Интерактивный отчёт / Interactive report](performance.html) · [CSV](performance/measurements.csv) · [JSON](performance/measurements.json)

Замер текущих примеров, без смешивания с прежними таблицами QSL. Windows 11 Pro x86-64 (10.0.26200), Intel Core i5-7400, 24 GiB / 2133 MHz; Rust 1.98.0 / LLVM 22.1.8; Criterion 0.8.2. Измерительный поток: CPU mask 0x4, above-normal; ОС не изолирована.

15 с прогрева + 10 с измерения на операцию, 200 выборок, 99% CI; 1024 заранее подготовленных записи. T — среднее Criterion в нс/операцию. M — исходные полезные байты, P=M/T — B/ns. Размер wire включает метаданные; он не используется как M.

Current examples only; older QSL measurements are separate. Warm-up 15 s, measurement 10 s, 200 samples, 99% CI, 1024 prepared records. T is Criterion's arithmetic mean in ns/op, M original useful bytes, and P=M/T is B/ns. Wire bytes include metadata and do not replace M.

Owned reads include construction and destruction; writes reuse prepared output unless labelled heap. Checked includes the generated validation and resource limit. Trusted assumes a prevalidated matching frame. Borrowed results do not copy payload bytes: their P is not memcpy bandwidth. Validation-only is not a full decode. RT uses one live uncontended lease.

"""
text+=f"Измерено {len(rows)} операций, снимков: {len(profiles)}. Основной кодек совпадает; каждая сравниваемая группа относится к одному бинарному файлу. Источники и профиль указаны для каждой строки JSON/CSV; одинаковые случаи не объединяются. / {len(rows)} operations across {len(profiles)} source profiles. Codec sources match, matched groups share a binary, and duplicate cases are never pooled.\n\n"
text+=f"Пиковая память JobObject: **{memory['peakJobBytes']/1048576:.2f} MiB**, лимит **8 GiB**. Peak job memory includes measured process descendants. Source SHA-256 before/after is identical.\n\n"
for group in order:
    selected=[r for r in rows if r["group"]==group]
    if not selected:continue
    text+=f"## {groups[group][0]} / {groups[group][1]}\n\n"
    text+="| Операция / Operation | Контракт | M, B | Wire, B | T, ns | 99% CI, ns | P, B/ns |\n|---|---|---:|---:|---:|---:|---:|\n"
    for r in selected:
        mode=r["case"].split("/")[-1]
        text+=f"| {r['operation']} / {mode} | {r['contract']} | {r['useful_bytes']:.1f} | {r['wire_bytes']:.2f} | {number(r['time_ns'])} | {number(r['time_low'])}–{number(r['time_high'])} | {r['p']:.4f} |\n"
    text+="\n"
text+="""## Как читать результаты / Interpretation

Строковый набор содержит ровно 1024 символа, но размер M указан в байтах UTF-8. Доверенное чтение String и checked-чтение выполняют разную работу. Новая CopyGuard тоже владеет выделенным буфером; это не замер reuse. Для фразы LZ выбрал raw-fallback: разница в 2 B обусловлена заголовком. Отдельный набор повторяющейся кириллицы демонстрирует настоящее LZ и сравнивается с собственной String-базой. Общий QPS сравнивается с обычным кодеком на одинаковых ASCII-данных, а не с UTF-8-группой.

Optional-поля — одинаковые u8. Для каждого N состояния равновероятны; M и wire являются средними по набору. Улучшение opt_table зависит от N и режима чтения; все измеренные варианты приведены, включая замедления. RT сравнивается с новой Vec; подготовленный выход дополнительно представлен numbers_8/write/raw_checked.

The UTF-8 set has exactly 1024 characters, while M counts encoded UTF-8 bytes. Fresh CopyGuard owns an allocated buffer; it is not reuse. The phrase set uses LZ raw fallback (2 B shorter header); the separate repeated-Cyrillic set measures genuine compression with its own String baseline. Whole QPS and its default baseline share ASCII data. Optional states are uniform and sizes are fixture means. All modes, including regressions, are retained. RT acquisition/release is included; reusable output has its own numbers_8 baseline.

Кардиограмма показывает 200 средних внутри пакетов Criterion. Это не отдельные задержки сообщений, не сеть и не p99. Для операции с разным контрактом значение P не означает одинаковый объём копирования.

## Воспроизводимость / Reproduction

[Исходники измеренного снимка](performance/measured-sources.zip) · [SHA-256 исходников](performance/source-before.json) · [Оригинальный Criterion JSON](performance/criterion-raw.zip) · [Лог](performance/run.log) · [Лимит и SHA-256 бинарного файла](performance/memory.json).

The source archive retains exact measured bytes; paths in manifests are relative to its root. The executable SHA-256 identifies the local release build, not a promise of byte-identical binaries from different source paths/toolchains. No new performance guarantee follows from this one machine and dataset.

```powershell
pwsh -ExecutionPolicy Bypass -File examples/tools/measure.ps1 -ResultDirectory examples/results/my-run
python examples/tools/build-report.py examples/results/my-run
```
"""
text+="\n### Снимки / Profiles\n\n"+"".join(f"- {p['id']}: {p['cases']} cases · [source](performance/{p['source_archive']}) · [raw](performance/{p['raw_archive']}) · [SHA-256](performance/{p['source_manifest']})\n" for p in profiles)
(examples / "PERFORMANCE.md").write_text(text,encoding="utf-8",newline="\n")
html=(examples/"tools/report-template.html").read_text(encoding="utf-8")
html=html.replace("__PAYLOAD__",json.dumps(report,ensure_ascii=False,separators=(",",":")).replace("<","\\u003c"))
(examples/"performance.html").write_text(html,encoding="utf-8",newline="\n")
# A conventional export accompanies the interactive view.
try:
 import matplotlib
 matplotlib.use("Agg")
 import matplotlib.pyplot as plt
 plt.rcParams.update({"font.family":"DejaVu Sans","font.size":9,"axes.spines.top":False,"axes.spines.right":False,"svg.fonttype":"none"})
 fig,axes=plt.subplots(2,3,figsize=(15,8.5),layout="constrained")
 panels=[("numbers_8","read","8 ranged u64 / read"),("strings_utf8_1024","read","UTF-8 phrase / LZ raw fallback"),("strings_utf8_1024_uniform","read","Repeated UTF-8 / genuine LZ"),("whole_qps_ascii_1024","read","1024 ASCII bytes / read"),("buffers","write","Output storage / write")]
 for ax,(group,op,title) in zip(axes.flat,panels):
  data=[r for r in rows if r["group"]==group and r["operation"]==op]
  labels=[r["case"].split("/")[-1].replace("_checked", " [checked]").replace("_trusted"," [trusted]") for r in data]
  y=list(range(len(data)));mean=[r["time_ns"] for r in data]
  bars=ax.barh(y,mean,color=["#c98442" if r["contract"]=="trusted" else "#136f85" for r in data],alpha=.88)
  ax.errorbar(mean,y,xerr=[[r["time_ns"]-r["time_low"] for r in data],[r["time_high"]-r["time_ns"] for r in data]],fmt="none",ecolor="#152535",capsize=3)
  ax.bar_label(bars,labels=[f"{v:.1f}" for v in mean],padding=3,fontsize=7.5);ax.margins(x=.26)
  ax.set_yticks(y,labels);ax.invert_yaxis();ax.set_title(title,loc="left",fontweight="bold");ax.set_xlabel("ns/op, mean and 99% CI");ax.grid(axis="x",alpha=.15);ax.set_axisbelow(True)
 ax=axes.flat[5];xs=[];ys=[]
 for n in [1,2,3,5,10]:
  a=next((r for r in rows if r["case"]==f"optional_N{n}/read/default_checked"),None)
  b=next((r for r in rows if r["case"]==f"optional_N{n}/read/opt_table_checked"),None)
  if a and b:xs.append(n);ys.append(a["time_ns"]/b["time_ns"])
 ax.plot(xs,ys,"o-",color="#136f85",linewidth=2);ax.axhline(1,color="#a66636",linestyle="--");ax.set_xticks(xs);ax.set_xlabel("optional u8 fields");ax.set_ylabel("default T / opt_table T");ax.set_title("opt_table / checked read",loc="left",fontweight="bold");ax.grid(alpha=.15)
 fig.suptitle("QSL examples · current measured build · i5-7400 · Criterion / 200 samples",fontsize=15,fontweight="bold")
 fig.savefig(dest/"overview.svg");fig.savefig(dest/"overview.png",dpi=170);plt.close(fig)
except ImportError:
 print("matplotlib unavailable: interactive report still generated")
paths=[p for p in dest.rglob("*") if p.is_file() and p.name!="SHA256SUMS.txt"]
(dest/"SHA256SUMS.txt").write_text("".join(hashlib.sha256(p.read_bytes()).hexdigest()+"  "+p.relative_to(dest).as_posix()+"\n" for p in sorted(paths)),encoding="utf-8",newline="\n")
print(f"Built report: {len(rows)} operations, source snapshot verified, peak job {memory['peakJobBytes']/1048576:.2f} MiB")
