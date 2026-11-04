# Строгое сравнение QSL / Strict QSL comparison

[Графики / Charts](report.html) · [Состояние результатов / Results](RESULTS.md) · [Методика / Method](METHOD.md) · [Безопасность RU](SECURITY.ru.md) · [Safety EN](SECURITY.en.md) · [Кандидаты RU](OPTIMIZATION_CANDIDATES.ru.md) · [Candidates EN](OPTIMIZATION_CANDIDATES.en.md)

## Русский

Стенд сравнивает обмен одними и теми же ограниченными данными через QSL, Cap’n Proto, FlatBuffers, Protobuf/prost, Bincode и настоящий typed gRPC/HTTP2. Это сравнение подготовленных нативных API и выбранных схем на Windows x86-64, а не рейтинг настроек по умолчанию или универсальный прогноз для серверов. QSL показан в обычном и специализированном вариантах. Оптимизации движка из отдельного списка не внесены.

**Numeric**: u64 id, восемь исходных u64 со значениями 0..4095, optional u8/u16. **Mixed**: те же значения, 1024 символа UTF-8 (1683 B) и массив 128 B. Все четыре сочетания optional-полей равномерно присутствуют в кольце из 1024 записей. Исходные полезные объёмы M: 73.5 / 1884.5 B на запись. P=M/T, B/ns. Wire-размер приведён отдельно.

Матрица: 60 случаев codec + 32 transport + 8 контролей = 100. Numeric: batch 1; Mixed: 1/4/16/64. Criterion: прогрев 15 s, измерение 10 s, 200 выборок, отдельный 99% CI. Подключения persistent; создание соединений вне замера. Чтение проверяет и потребляет все значения, строки и массивы. TCP — явный framed-протокол стенда, не Cap’n Proto RPC. gRPC имеет unary и bidirectional stream. Это одиночный request/ack, не тест максимальной конкурентной нагрузки.

Основной исполняемый файл не содержит allocator-hook и опроса метрик в горячем цикле. Внешний процесс собирает показатели на границах случая. Перед каждым случаем — 3 последовательных секунды с загрузкой каждого ядра ≤5%. После случая — известные посторонние процессы ≤2% мощности CPU и консервативный остаток после DPC/IRQ ≤2%. Источники/бинарный файл фиксируются хэшами. Результат со sampler, изменёнными исходниками, ошибкой или шумом исключается из основного сравнения. При невозможности простоя проход останавливается, пороги не снижаются.

Лимит измеряемого JobObject — **8 GiB**, включая потомков. Peak job commit и peak working set различаются. CPU/cycles/peak RAM относятся ко всему случаю; их нельзя безусловно делить на количество итераций Criterion. I-cache misses недоступны: CPUID сообщает лишь ёмкость. Контроль allocator и отдельные request-level RTT не подменяют основной замер.

### Команды (PowerShell 7, из этой папки)

```powershell
Set-Location C:\Users\Guild\Desktop\dev-qsl\benchmarks\protocols
pwsh -NoProfile -File tools/setup-tools.ps1
$env:RUSTFLAGS='-C target-cpu=native'
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
pwsh -NoProfile -File tools/run.ps1 -Output results/strict-final
```

Перед запуском свернуть этот чат/браузеры и завершить тяжёлые фоновые задачи. Полный проход занимает примерно 45–60 минут при устойчивом простое. Стенд сам проверяет простой; факт свёрнутого окна его не заменяет. Новому проходу нужна новая папка результатов. Не выполнять сборки и диагностику параллельно с основным измерением.

Узкий контроль (без запуска всей матрицы):

```powershell
pwsh -NoProfile -File tools/run.ps1 -Filter '^codec/mixed/read/b16/(qsl_default|qsl_optimized)$' -Output results/narrow-control
```

Отдельные диагностические проходы и контроль влияния сборщика:

```powershell
pwsh -NoProfile -File tools/probe.ps1 -Program allocation_probe -Output results/allocation-final
pwsh -NoProfile -File tools/probe.ps1 -Program latency_probe -Output results/latency-final
pwsh -NoProfile -File tools/collector-control.ps1
python -B tools/build-report.py
```

Для экспорта PNG/SVG нужен matplotlib; HTML/JSON/CSV формируются без него. Отчёт автономен: RU/EN, фильтры сценария/операции/batch, T/P/RAM/CPU, 200 точек Criterion, масштабирование batch, аллокации и отдельные RTT. Отклонённые данные скрыты по умолчанию. Все исходные выборки и причины отказа сохраняются в `report/raw.zip`. `report/data.json` определяет актуальную полноту: отсутствие принятого случая означает отсутствие результата, а не нулевое время.

## English

This Windows x86-64 harness compares identical bounded values through native QSL, Cap’n Proto, FlatBuffers, Protobuf/prost, Bincode and typed gRPC/HTTP2. It measures prepared APIs with explicitly optimized schemas, not default settings or a universal server ranking. Both ordinary and specialized QSL are retained. No engine optimization from the candidate list is applied.

Numeric contains a u64 id, eight original u64 values in 0..4095 and optional u8/u16. Mixed adds 1024 UTF-8 scalars (1683 B) and 128 B data. A 1024-record fixture ring equally covers all four optional states. Original useful M is 73.5 / 1884.5 B per record; P=M/T in B/ns. Encoded wire size is separate.

The 100-case matrix contains 60 codec profiles, 32 transport profiles and 8 controls. Numeric uses batch 1; Mixed uses batches 1/4/16/64. Criterion uses 15 s warmup, 10 s measurement, 200 samples and marginal 99% CIs. Persistent connections exclude setup. Checked reads fully consume every value and text/data byte. Framed TCP is the harness application protocol, not Cap’n Proto RPC. Native gRPC has unary and bidirectional streaming profiles. Single request/ack measurements do not establish maximum-concurrency capacity.

Run the PowerShell 7 commands above from this directory. Minimize this chat/browser and stop heavy background work first. Build/test before timing, never concurrently. A full quiet pass takes approximately 45–60 minutes. Each new pass needs a new output directory. A narrow `-Filter` can select two relevant cases instead.

The timing executable contains neither allocation instrumentation nor metrics polling in its hot loop. The external driver collects boundary metrics. Every case requires three consecutive one-second intervals with every CPU core ≤5%; post-run known foreign CPU and conservative non-DPC/IRQ residual must each stay ≤2% of total CPU capacity. Source and binary hashes, exit status and sampler state gate acceptance. An idle failure stops the pass without weakening thresholds.

A JobObject caps the measured process and descendants at 8 GiB. Peak commit differs from peak resident working set. CPU/cycle/peak values cover the full case, not one operation. I-cache misses remain unavailable; CPUID provides capacity only. Allocation and request-level RTT probes run separately. Collector A/B/A controls are diagnostic and never pooled with the main run. Controls are never subtracted from timings.

The self-contained RU/EN report provides T/P/CPU/RAM filters, Criterion sample waveforms, batch scaling, allocation counts and separate RTTs. Rejected observations are hidden by default. Missing qualified cases have no result, not zero time. Raw samples and rejection evidence are archived; source and binary digests preserve provenance. Optional matplotlib exports PNG/SVG; HTML/JSON/CSV need no plotting package.

### Фоновое ожидание / Background idle waiting

```powershell
pwsh -NoProfile -File tools/start-waiting.ps1 -MaxHours 4
```

Окно скрыто; интерфейс чата можно закрыть. Контроллер ждёт простой и повторяет только непринятые случаи, сохраняя каждый проход отдельно. Основное ожидание/измерение ограничено четырьмя часами. После полной матрицы выполняются отдельные диагностические проходы. Локальные `report.html` и `report/data.json` обновляются между проходами. GitHub автоматически не изменяется. Состояние: `results/background-state.json`; журналы: `results/background-*.log`. Изменение исходников останавливает ожидание.

Остановка без убийства процесса: создать файл `results/STOP`; текущий случай завершится, новые не начнутся. Для нового запуска удалить этот файл явно. Не запускать два контроллера одновременно.

The hidden controller waits for idle, retries only unqualified cases, preserves every pass and refreshes local reports between passes. Main waiting/measurement is bounded by four hours; separate diagnostics follow a complete matrix. It never publishes to Git automatically. State and logs are in `results/`. Create `results/STOP` to stop after the current case; explicitly remove it before another launch. Source changes stop the controller. Do not run two controllers concurrently.
