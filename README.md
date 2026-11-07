> **Текущая ветка main:** общее исполнение и контракты схем вынесены в [qsl-core](qsl-core). Парсер и Rust-генератор остаются отдельными. Переменные строки: String, &str, StackString и InlineString; new/reuse — режимы методов. Эти изменения исходников следуют после опубликованного 0.2.0. [Архитектура](docs/architecture.ru.md) · [Переход](docs/migration-0.3.0.ru.md).

# QSL 0.2.0

[Русский](#русский) · [English](#english) · [Release v0.2.0](https://github.com/zadr0/QSL/releases/tag/v0.2.0)

## Русский

QSL — язык описания заранее согласованных данных и генератор специализированных бинарных кодеков Rust. При сборке схема задаёт порядок полей, диапазоны, длины, владение и проверки; приложение получает конкретные структуры и методы. Разбор схемы не входит в горячий путь.

Это узкий инструмент для управляемых внутренних протоколов, телеметрии и повторяемых записей. Условия совместимости, TLS, маршрутизацию и внешнюю IPC-память обеспечивает приложение. QSL предоставляет формат и средства работы с подготовленными буферами; LLVM-backend и QBProto-рукопожатие пока не реализованы.

```toml
[dependencies]
qsl = "=0.2.0"

[build-dependencies]
qsl = "=0.2.0"
```

```qsl
&User[safety,aggressive,optionals=bitmap,opt_table] {
    u64 id;
    str(1..100) name;
    u8? status;
}
```

Обязательные поля находятся перед опциональными. Длина строки измеряется в байтах UTF-8. Здесь name использует ограниченный встроенный буфер, status — сгенерированный optional-доступ; объект создаётся через try_new. Без aggressive переменная COPY-строка обычно хранится в String. safety включает checked-десериализацию; явные unchecked-методы сохраняются.

| Возможность | Контракт |
|---|---|
| default / opt_table | Match или статическая функция по optional-маркеру; обычный теговый формат сохраняется |
| bitmap | Только optional-поля, маска 8–128 бит, обход установленных битов |
| predict_seq | Индекс по длине; не управляет порядком BM |
| qps=Q2/Q4/Q8/Q16 | Обратимая числовая упаковка; по умолчанию Q16 |
| Строки | String, ограниченное встроенное хранение, &str / &[u8], отдельное QSL-LZ |
| Память | serialize_to, deserialize_into, SerOut, ограниченные RT-слоты |
| Native-layout | COPY и проверяемый LIFETIME; именованные регионы создают вложенные типы |
| Пользовательские типы | QslCustom, проверяемые qsl_primitive! / qsl_native_primitive!, Rust-шаблоны |
| Безопасность | Курсор, диапазоны, UTF-8, контроль распаковки, MAX_DATA_SIZE=65535 |
| Компилятор | qsl-lexer → общий QslSchemaBuilder → qsl-interpreter |

**Переход с 0.1:** BM меняет wire-формат, обе стороны обновляются согласованно. Cargo-feature native, Windows-сокеты/IPC-регионы и windows-sys удалены; обычный serialize возвращает Vec<u8>, serialize_to принимает срез. Нативный layout и SerOut от Windows не зависят. [Полный переход и ограничения](docs/migration-0.2.0.ru.md).

STACK_LIMIT по умолчанию — 256 B данных на встроенный буфер. Превышение использует String/Vec при сохранении кодека; фиксированный native-layout требует явно достаточного лимита. Атрибуты опциональны; их наличие не гарантирует ускорение. Хэши учитывают формат, но не заменяют согласование версий или аутентификацию.

[Полный интерактивный справочник](https://zadr0.github.io/QSL/index.html) · [Архитектура](docs/architecture.ru.md) · [API](docs/API.ru.md) · [Безопасность](docs/safety.ru.md) · [Примитивы и регионы](docs/custom-types.ru.md) · [Примеры](examples/README.md) · [Заметки выпуска](docs/releases/0.2.0.ru.md).

Проект проверяется на Windows 11 x86-64, i5-7400, 24 GiB / 2133 MHz. Старые измерения сохраняют исходные версии. Прогон 7 ноября не принят как строгий из-за загрузки CPU; новое ускорение им не доказано. Цель 25 нс для 1024 символов UTF-8 не объявляется достигнутой.

[MIT](LICENSE) · [crates.io](https://crates.io/crates/qsl/0.2.0) · [docs.rs](https://docs.rs/qsl/0.2.0/qsl/)

## English

QSL is a schema language and specialized Rust binary-codec generator for data described in advance. Build-time analysis resolves field order, domains, lengths, ownership and validation into concrete structures and methods. Schema parsing is outside the hot path.

It targets controlled internal protocols, telemetry and repeated records. Applications provide version negotiation, TLS, routing and external IPC storage. QSL supplies the format and caller-prepared buffer interfaces. An LLVM backend and QBProto handshake are planned, not implemented.

```toml
[dependencies]
qsl = "=0.2.0"

[build-dependencies]
qsl = "=0.2.0"
```

```qsl
&User[safety,aggressive,optionals=bitmap,opt_table] {
    u64 id;
    str(1..100) name;
    u8? status;
}
```

Required fields precede optionals; string bounds count UTF-8 bytes. Here name uses bounded inline storage and status has generated optional accessors; construct with try_new. Without aggressive, a variable COPY string normally uses String. safety enables checked deserialization; explicit unchecked methods remain available.

| Capability | Contract |
|---|---|
| default / opt_table | Match or a static optional-reader table; ordinary tag bytes are preserved |
| bitmap | Optional-only 8–128-bit masks and set-bit traversal |
| predict_seq | Length-derived index; inactive under BM |
| qps=Q2/Q4/Q8/Q16 | Reversible scalar packing; Q16 by default |
| Strings | String, bounded inline storage, &str / &[u8], independent QSL-LZ |
| Memory | serialize_to, deserialize_into, SerOut and bounded RT slots |
| Native layout | COPY and checked aligned LIFETIME; named regions create nested types |
| Custom types | QslCustom, validating scalar-wrapper macros and reusable Rust templates |
| Safety | Cursor bounds, ranges, UTF-8, decompression limits, MAX_DATA_SIZE=65535 |
| Compiler | qsl-lexer → one QslSchemaBuilder → qsl-interpreter |

**Migrating from 0.1:** BM changes wire format and requires coordinated peer migration. The native Cargo feature, Windows sockets/IPC regions and windows-sys are removed. Ordinary serialize returns Vec<u8>; serialize_to takes a mutable slice. Native layout and SerOut do not depend on Windows. [Complete migration and constraints](docs/migration-0.2.0.en.md).

STACK_LIMIT defaults to 256 data bytes per inline buffer. Larger owned buffers use String/Vec while retaining their codec; fixed native layouts require an explicitly sufficient limit. Attributes are opt-in and do not guarantee a speedup. Schema hashes describe format identity, not version negotiation or authentication.

[Interactive reference](https://zadr0.github.io/QSL/en.html) · [Architecture](docs/architecture.en.md) · [API](docs/API.en.md) · [Safety](docs/safety.en.md) · [Custom types](docs/custom-types.en.md) · [Examples](examples/README.md) · [Release notes](docs/releases/0.2.0.en.md).

Recorded hardware is Windows 11 x86-64, i5-7400 and 24 GiB / 2133 MHz. Historical measurements retain their source snapshots. The November 7 pass is rejected as a strict result because of CPU activity and establishes no new speedup. The 25 ns target for 1024 UTF-8 characters is not claimed achieved.

[MIT](LICENSE) · [crates.io](https://crates.io/crates/qsl/0.2.0) · [docs.rs](https://docs.rs/qsl/0.2.0/qsl/)
