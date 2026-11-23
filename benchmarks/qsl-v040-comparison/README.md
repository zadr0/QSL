# QSL 0.4.0: воспроизводимое сравнение кодеков

Изолированный стенд использует опубликованный qsl = "=0.4.0" из crates.io. Исходники самой библиотеки не изменены для сравнения.

## Отчёты

| Отчёт | Что измерено |
|---|---|
| [HTML с вкладками](dashboard.html) | Кодеки, полный HTTP API, TCP Heartbeat/Events, функции QSL и калькулятор трафика |
| [Прикладные сценарии](REALISTIC.md) | Полиморфные вложенные сообщения, одни входы для шести вариантов, k6 и постоянные TCP-соединения |
| [Трансмутация и строки](QSL-FEATURES.md) | Чтение по полям, copy, overlay, String, StackString и QSL-LZ |
| [Одинаковые структуры](SAME-STRUCTURES.md) | По одному фиксированному baseline QSL / Protobuf / Bincode / Cap’n Proto на общем наборе данных и с одинаковым владением |
| [Только QSL](QSL-ONLY.md) | default, compact, alignment, pointing, group/BMI2, PS, OT, BM, String/StackString/&str |
| [Все режимы Dyn](DYN.md) | line 16/32/64, index(dynamic) 32/64, index(strict) 32/64; compact для индексов; owned и borrowed |
| [Автоматические раскладки](LAYOUT.md) | Rust-смещения, размер и выравнивание, изменения кадра, индексы памяти и скорости для `auto` и `auto[wire=1]` |
| [k6 / HTTP](K6.md) | Запись и чтение трёх структур для четырёх протоколов и всех семи Dyn-режимов; closed и open model |
| [Полная исходная матрица](RESULTS.md) | Все основные варианты, размеры кадров, аллокации и ограничения |
| [HTML основной матрицы](report.html) | Графики прежних изолированных замеров чисел, optional-полей и строк |

Разделы сравнения одинаковых структур и вариантов QSL являются разными срезами одного полного основного прогона. Dyn измерен отдельным исполняемым файлом и отдельными проходами. k6 выполняется после всех микробенчмарков; нагрузочные тесты не идут одновременно с ними.

Сравниваются QSL 0.4.0 / qsl-core 0.2.0, Protobuf через Prost 0.14.4, Cap'n Proto Rust 0.27.3 (обычный unpacked frame), Bincode 2.0.1. Все версии зависимостей закреплены Cargo.lock. Cap'n Proto schema compiler: 1.5.0. Для выбранного профиля нужен x86-64 CPU с BMI2. Бинарник собран для конкретного CPU.

## Данные и варианты

- numeric: четыре логических u32 с одинаковым диапазоном 0..4095. QSL default, compact, compact+alignment, pointing, compact+group/BMI2; Protobuf varint/fixed32; Bincode varint/fixint; дополнительно Bincode и Cap'n Proto с физическими u16, сохраняющими тот же домен значений.
- optionals: id:u32 и a:Option<u8>, b:Option<u16>, c:Option<u32>. Все восемь масок присутствия равновероятны; присутствующий ноль отличается от отсутствия. QSL default, OT, PS, PS+OT, BM, BM+OT.
- text_32, text_256, text_1024: id:u32 и UTF-8 строка соответствующей длины в байтах. QSL String, StackString<1024> при STACK_LIMIT=2048 и lifetime/&str.

Для выбранных optional-полей остаточная длина равна сумме 1, 2 и 4 байт присутствующих значений; все восемь сумм различны. Поэтому PS может определить присутствие из длины. Это условие не переносится автоматически на другие схемы.

Кольцо из 256 записей построено детерминированно. Числа и id меняются между записями, строки содержат ASCII, кириллицу, греческие, китайские символы и emoji. Подготовка исходных объектов и буферов исключена из замеров. Проверки восстанавливают каждое значение, а не только размер сообщения.

## Контракт операций

write_prepared: запись проверенных схемой значений в заранее выделенный выходной буфер. QSL использует serialize_checked_to; Bincode encode_into_slice; Prost encode; Cap'n Proto заполняет новый message builder на повторно используемом scratch-буфере и пишет стандартный frame с segment table. Создание builder и очистка использованной его части входят во время Cap'n Proto. Его служебные аллокации учитываются отдельно.

read_owned: проверяемое чтение всех полей, создание самостоятельного результата, одинаковый checksum всех скаляров/байтов строки и уничтожение результата. QSL StackString владеет строкой без heap; для Cap'n Proto текст материализуется в String.

read_borrowed: отдельная категория для QSL lifetime, Bincode borrow_decode_from_slice и полного обхода Cap'n Proto reader. Она не ранжируется как эквивалент чтения с владением.

Конкурентам добавлены такие же проверки числового диапазона и максимальной длины строки. Измеряются валидные входы. Форматы по-разному трактуют неизвестные поля, недостающие proto3-поля и расширение схем; это не утверждение об одинаковой модели эволюции или о полной эквивалентности проверки повреждённых данных.

Время включает общий consumer. Контроли consumer-only сохранены отдельно, их стоимость не вычитается. std::hint::black_box защищает входы, выходные байты и прочитанные объекты от удаления компилятором.

Размер frame включает собственные метаданные каждого формата, но не транспортный TCP/RPC заголовок. Для переменных размеров показаны среднее, минимум и максимум по тому же кольцу.

Аллокации измерены отдельным probe-бинарником. Глобальный счётчик аллокатора не включён в исполняемый файл Criterion.

allocated_bytes — сумма размеров запросов alloc/alloc_zeroed/realloc, а не пиковый RSS. Память native-объектов и scratch описана отдельно от этого счётчика.

## Измерение

Один и тот же release-бинарник, target-cpu=native, thin LTO, codegen-units=1, логическое ядро 2, приоритет процесса AboveNormal. 200 выборок, прогрев 1 с, измерение 3 с, bootstrap 10000, CI 99%. Два независимых прохода в прямом и обратном порядке. Замеры выполняются в обычной Windows; фоновая нагрузка и условия записаны в results/released/environment.json и results/released/runs.json. План питания не изменяется.

В таблицах используется среднее двух медиан; исходные медианы и CI каждого прохода находятся в CSV/JSON. Отрезки на графиках показывают различие медиан между проходами, а не объединённый доверительный интервал. Малые различия, особенно при перекрывающихся CI или заметном дрейфе, не следует считать доказанным превосходством.

Это измерение прогретого набора данных на одной машине, без сетевого обмена. Результаты не являются универсальными показателями для произвольных CPU, размеров, распределений и схем.

## Воспроизведение (PowerShell)

Команды рассчитаны на Windows x86-64, PowerShell, Rust и Node.js 20 или новее. Измеренный профиль использует BMI2 и четыре логических ядра. Выполняйте команды из этой папки. Для k6 установите Grafana k6 перед запуском HTTP-серии.

~~~powershell
./tools/setup-capnp.ps1
$env:CAPNP = (Resolve-Path ./tools/bin/capnp.exe).Path
$env:RUSTFLAGS = '-C target-cpu=native'
cargo test --release --locked
cargo bench --locked --bench comparison --no-run --message-format=json > build-artifacts.jsonl
New-Item -ItemType Directory -Force results/released | Out-Null
cargo run --release --locked --bin probe > results/released/profiles.json
./tools/run-released.ps1
npm ci --prefix tools --ignore-scripts
node tools/report.mjs
~~~

Во время измерения не запускайте сборки и другие CPU-тесты. Существующий Cargo cache позволяет использовать --offline. При отсутствии зависимостей сначала выполните cargo fetch --locked.

В `results/released` и `results/dyn` находятся сырые выборки Criterion, оценки и CI, CSV и сведения о CPU. В `results/k6` находятся JSON каждого испытания, CSV и метаданные среды. Хэши связывают измерения с исходниками и бинарником. Генераторы отчётов после повторного запуска используют метаданные новых замеров.

Основные таблицы используют завершённые проходы из `results/released/runs.json`. Перед каждым проходом Criterion проверяет общий CPU ≤20%. Фактическая фоновая нагрузка во время каждого прохода записана отдельно. Предварительные прогоны с повышенной нагрузкой исключены из опубликованных данных. Для HTTP-замеров k6 фон записан по каждому испытанию.

## Dyn и k6: воспроизведение

После сборки основного стенда в том же PowerShell:

~~~powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
cargo test --release --locked --manifest-path dyn/Cargo.toml
cargo bench --locked --manifest-path dyn/Cargo.toml --bench comparison --no-run --message-format=json > dyn/build-artifacts.jsonl
New-Item -ItemType Directory -Force results/dyn | Out-Null
cargo run --release --locked --manifest-path dyn/Cargo.toml --bin probe > results/dyn/profiles.json
cargo build --release --locked --manifest-path http/Cargo.toml
./tools/run-dyn.ps1
./tools/run-k6.ps1
node tools/report.mjs
node tools/extra-reports.mjs
node tools/k6-report.mjs
~~~

Нужен установленный Grafana k6; измеренная версия — 2.2.0. Сервер слушает только 127.0.0.1:7340, затем автоматически завершается. Сервер и k6 закрепляются на разных логических ядрах одной машины. В k6 измеряются весь HTTP-путь и проверка ответа клиентом; это отдельная величина от наносекунд кодека. Нагрузки на внешний сайт нет.

run-k6.ps1 отказывается перезаписывать уже существующие испытания. Перед новым воспроизведением сохраните старую папку results/k6 отдельно. Старые результаты также нужно сохранять перед повторным полным Criterion-проходом: опубликованные сырые данные должны оставаться неизменными.

Dyn включает 32 профиля и 60 случаев: семь режимов × numeric/optionals/text-owned/text-borrowed, четыре дополнительных numeric compact и три consumer-контроля. В k6 — 33 профиля: 12 базовых и 21 Dyn; 136 closed-испытаний и 8 open-испытаний. Не измеряются все произвольные сочетания атрибутов, неизвестные поля, schema evolution, повреждённые входы под нагрузкой, QSL-RT или производственная сеть.

## Воспроизведение раскладок

В той же среде PowerShell и без параллельного k6 выполните:

~~~powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
$env:RUSTFLAGS = '-C target-cpu=native'
cargo test --release --locked --manifest-path layout/Cargo.toml
cargo bench --locked --manifest-path layout/Cargo.toml --bench comparison --no-run --message-format=json > layout/build-artifacts.jsonl
New-Item -ItemType Directory -Force results/layout | Out-Null
cargo run --release --locked --manifest-path layout/Cargo.toml --bin probe > results/layout/profiles.json
./tools/run-dyn.ps1 -Suite layout -Prefix layout-
node tools/layout-report.mjs
~~~

## Воспроизведение прикладных сценариев и функций QSL

Сохраните старые `results/realistic` и `results/qsl-features` в другой папке перед новым запуском. Для первого запуска с пустым Cargo cache загрузите зависимости командой `cargo fetch --locked --manifest-path realistic/Cargo.toml` и аналогично для `qsl-features/Cargo.toml`. В каталоге `tools/bin` нужен компилятор Cap’n Proto из предыдущего шага.

~~~powershell
./tools/run-realistic.ps1
./tools/run-dyn.ps1 -Suite qsl-features -Prefix qsl-features-
node tools/scenario-report.mjs
node tools/serve-reports.mjs 7360
~~~

[Локальный HTML](http://127.0.0.1:7360/dashboard.html) доступен при запущенном сервере. Можно также открыть скачанный `dashboard.html` напрямую. Оба транспорта используют loopback. Изолированные замеры, сокеты и k6 выполняются последовательно.

## Обозначения QSL

| Метка в отчёте | Атрибуты схемы |
|---|---|
| qsl_default | safety |
| qsl_compact | safety, compact |
| qsl_compact_alignment | safety, compact, alignment |
| qsl_pointing | safety, aggressive[pointing=true] |
| qsl_compact_group_bmi2 | safety, compact, aggressive[group=true]; CPU_PROFILE=kaby_lake |
| qsl_ot | safety, opt_table |
| qsl_ps | safety, predict_seq |
| qsl_ps_ot | safety, predict_seq, opt_table |
| qsl_bm | safety, optionals=bitmap |
| qsl_bm_ot | safety, optionals=bitmap, opt_table |
| qsl_heap | safety; String |
| qsl_inline | safety, aggressive; StackString<1024> |
| qsl_lifetime | safety, lifetime; &str |

Вариант default есть отдельно для чисел и optional-схемы, поэтому всего измеряется 14 различных схем QSL.

Источники спецификаций: [Prost](https://github.com/tokio-rs/prost), [Bincode 2.0.1](https://docs.rs/bincode/2.0.1/bincode/), [формат Cap’n Proto](https://capnproto.org/encoding.html). Подробная версия QSL и checksums закреплены в Cargo.lock; библиотека не изменялась для этого сравнения.

Дополнительная проверка нестабильных случаев: ./tools/confirm.ps1 (2 с прогрева, 5 с измерения, 200 выборок, два порядка). Исходные тайминги не заменяются и не смешиваются с этими подтверждениями.
