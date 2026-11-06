# QSL

[Быстрый старт RU](docs/quickstart.ru.md) · [Quick start EN](docs/quickstart.en.md)

[Русский](#русский) · [English](#english) · [Interactive documentation](docs/index.html)

## Русский

QSL — язык описания данных и генератор специализированного Rust-кода для бинарного обмена. Исходная задача — обмен заранее описанными данными. Обе стороны знают поля, порядок, диапазоны и кодеки до запуска приложения.

Схема анализируется при сборке. QSL рассчитывает границы, битовые ширины, регистровые группы, таблицы и идентичность формата, затем генерирует конкретные методы. Значения и длины могут меняться в runtime; разбор самой схемы в горячий путь не входит.

### Установка

```toml
[dependencies]
qsl = "=0.1.1"

[build-dependencies]
qsl = "=0.1.1"
```

[Быстрый старт из четырёх файлов](docs/quickstart.ru.md). QSL нужен в build.rs для генерации и в приложении для исполнения.

### Где он применим

QSL рассчитан на управляемый обмен между согласованными компонентами: внутренние протоколы, IPC, телеметрия и повторяемые записи. Его узость — известный при сборке формат и явные инварианты памяти. Разработчик выбирает владение, ограничения, проверку входа и подготовку буферов. Универсальная маршрутизация, TLS, согласование версий и QBProto-рукопожатие не возникают из схемы автоматически.

~~~qsl
&User[safety,aggressive] {
    u64 id;
    str(1..100) name;
    u8? status;
}
~~~

Обязательные поля всегда перед опциональными. Здесь name — встроенная COPY-строка до 100 **байтов UTF-8**; status читается методом status(), изменяется через set_status(...), объект создаётся через try_new(...). Без aggressive переменная строка обычно становится String. Без safety checked-десериализация не генерируется.

### Что можно выбрать

| Задача | Механика |
|---|---|
| Сократить повторную работу чтения | Автоматические точные регистровые группы |
| Управлять владением | String, встроенный буфер, прямой &str / &[u8], фиксированные массивы |
| Упаковать самостоятельные числа без потерь | qps=Q2/Q4/Q8/Q16, по умолчанию Q16 |
| Сжать строку | Отдельный QSL-LZ; строки, массивы и длины не квантуются QPS |
| Читать optional-поля | predict_seq: индекс по длине; opt_table: тег → читатель |
| Контролировать недоверенный вход | safety, единый курсор, MAX_DATA_SIZE = 65535 |
| Готовить память | serialize_to / deserialize_into, RT-слоты, loyal_cache |
| Работать с нативным layout | Отдельные COPY и LIFETIME Transmutate, вложенные регионы |
| Переиспользовать описание | Rust qsl_macro / schem_macro с вложенным раскрытием полей |

Атрибуты опциональны. STACK_LIMIT по умолчанию **256 B данных на встроенный буфер**, строгий даже для явно выбранных inline/LZ. Сверх лимита COPY использует String/Vec; нативный фиксированный Transmutate требует явно достаточного лимита. RT ограниченных схем ждёт освобождения слота. Trusted-методы требуют корректного кадра и памяти. Низкоуровневые trait-методы serialize теперь unsafe; безопасный общий выход — serialize_checked.

При изменении формата стороны обновляются согласованно. Строгий и лёгкий хэши вычисляются при анализе; xxHash не является криптографической гарантией. Скорость зависит от схемы, архитектуры, размера рабочего набора и выбранного контракта.

[Контракт и переход RU](docs/modernization-2026-11-06/MODERNIZATION.ru.md) · [Полное руководство](docs/guide.ru.html) · [Рабочие примеры](examples/README.md) · [Точечные замеры с графиками](qsl-bench/modernization/report.html)

Сохранённые замеры 6 ноября: String 130,38 нс и встроенная строка 63,20 нс для 2048 B UTF-8; цель 25 нс не достигнута. Числовой перепрогон: default 2,788 нс / 24 B и Q16 4,304 нс / 5 B; ускорение не подтверждено. [Снимки и интервалы](qsl-bench/modernization/results/2026-11-06-register-width/README.md) относятся к измеренным исходникам. Сравнение протоколов — исторические 16/100.

## English

QSL is a data-description language and a specialized Rust code generator for binary exchange. Its original task is exchanging data described in advance. Both peers know field order, domains and codecs before application startup.

At build time QSL analyzes bounds, bit widths, register groups, tables and wire identities, then emits concrete methods. Runtime values and lengths remain dynamic; schema interpretation is outside the hot path.

### Installation

```toml
[dependencies]
qsl = "=0.1.1"

[build-dependencies]
qsl = "=0.1.1"
```

[Four-file quick start](docs/quickstart.en.md). Both dependencies are required: build.rs generates the codec and the application uses it.

### Where it fits

QSL targets controlled exchange between coordinated components: internal protocols, IPC, telemetry and repeated records. Its specialization is a build-time-known format with explicit memory invariants. Developers choose ownership, bounds, input checking and prepared storage. Routing, TLS, version negotiation and a QBProto handshake are not automatically supplied by a schema.

~~~qsl
&User[safety,aggressive] {
    u64 id;
    str(1..100) name;
    u8? status;
}
~~~

Required fields always precede optionals. Here name is an inline COPY string bounded by 100 **UTF-8 bytes**. Read status through status(), change it through set_status(...), and construct the record through try_new(...). Without aggressive, variable owned strings normally use String. Without safety, checked deserialization is not generated.

### Available choices

| Goal | Mechanism |
|---|---|
| Reduce repeated read work | Automatic exact register groups |
| Select ownership | String, inline storage, direct &str / &[u8], fixed arrays |
| Pack scalar numbers losslessly | qps=Q2/Q4/Q8/Q16; Q16 by default |
| Compress a string | Independent QSL-LZ; QPS never quantizes strings, arrays or lengths |
| Read optional fields | predict_seq: length index; opt_table: tag → reader |
| Control untrusted input | safety, one cursor, MAX_DATA_SIZE = 65535 |
| Prepare storage | serialize_to / deserialize_into, RT slots, loyal_cache |
| Use native layout | Separate COPY/LIFETIME Transmutate and nested regions |
| Reuse definitions | Nested Rust qsl_macro / schem_macro field expansion |

Attributes are opt-in. STACK_LIMIT defaults to **256 data bytes per inline buffer**, including explicit inline/LZ requests. Larger COPY buffers use String/Vec; fixed native Transmutate requires an explicitly sufficient limit. Bounded RT pools wait for a free slot. Trusted pointer methods require valid frames and correct memory. Low-level serialize trait methods are now unsafe; serialize_checked is the general safe output API.

Wire changes require coordinated peers. Strict and light hashes are computed during analysis; xxHash is not cryptographic. Performance depends on the schema, architecture, working-set size and checked/trusted contract.

[Contract and migration EN](docs/modernization-2026-11-06/MODERNIZATION.en.md) · [Complete guide](docs/guide.en.html) · [Runnable examples](examples/README.md) · [Focused measurements and charts](qsl-bench/modernization/report.html)

Recorded November 6 runs: String 130.38 ns and inline ownership 63.20 ns for 2048 UTF-8 bytes; the 25 ns target was not met. Numerical rerun: default 2.788 ns / 24 B and Q16 4.304 ns / 5 B; no speedup was established. [Snapshots and intervals](qsl-bench/modernization/results/2026-11-06-register-width/README.md) belong to their measured sources. The protocol comparison remains a historical 16/100 result.

Лицензия: [MIT](LICENSE).
