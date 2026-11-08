# QSL: быстрый старт

[Русский](quickstart.ru.md) · [English](quickstart.en.md)

QSL — язык описания данных и генератор специализированного Rust-кода для бинарного
обмена. Стороны заранее согласуют поля, порядок, диапазоны, длины и кодеки.
При сборке анализ схемы превращает эти условия в конкретные структуры и методы.

QSL рассчитан на внутренние протоколы, IPC, телеметрию и повторяющиеся записи
с известным форматом. Разработчик задаёт ограничения памяти, владение и проверку
входа; приложение организует согласование версий, маршрутизацию и TLS.

## Первый запуск

Нужен Rust 1.98 или новее. Релиз проверяется на Windows 11 x86-64.
Создайте приложение через cargo new qsl-demo и добавьте четыре файла.
В Cargo.toml сохраните секцию package, созданную Cargo.
Версия qsl в зависимостях приложения и сборки должна совпадать.

### 1. Cargo.toml

```toml
[dependencies]
qsl = "=0.3.0"

[build-dependencies]
qsl = "=0.3.0"
```

### 2. user.qsl

```qsl
&User[safety] {
    u64 id;
    str(1..100) name;
}
```

### 3. build.rs

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    qsl::build_qsl_files(&["user.qsl"], &[], out)?;
    Ok(())
}
```

### 4. src/main.rs

```rust
include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs"));

fn main() -> qsl::QslResult<()> {
    let user = User {
        id: 42,
        name: "Ada".into(),
    };
    let bytes = user.serialize_checked()?;
    let restored = User::deserialize_checked(&bytes)?;
    assert_eq!((restored.id, restored.name.as_str()), (42, "Ada"));
    println!("{}: {} ({} bytes)", restored.id, restored.name, bytes.len());
    Ok(())
}
```

Выполните cargo run. Ожидаемый вывод:

```text
42: Ada (15 bytes)
```

Схема описывает u64 и String длиной 1–100 байтов UTF-8. build.rs создаёт qsl_gen.rs
в OUT_DIR, приложение подключает сгенерированный тип и вызывает его проверенные методы.

## Основные правила

- Обязательные поля располагаются перед опциональными.
- Длины строк измеряются в байтах UTF-8, массивов — в элементах.
- safety включает checked-чтение. Unsafe API требует корректного кадра и памяти.
- MAX_DATA_SIZE по умолчанию равен 65535 B полезных данных и ограничивает входной кадр.
- STACK_LIMIT — 256 B данных на встроенный буфер. Большие обычные COPY-строки и
  массивы используют String/Vec; фиксированному Transmutate нужен достаточный лимит.
- Сжатый размер не доказывает длину результата: LZ проверяет каждый выходной токен.
- qps упаковывает числа без потерь; строки, массивы и длины сохраняют свои кодеки.
- predict_seq вычисляет индекс по длине, opt_table выбирает читателя по тегу.
- Атрибуты опциональны; ограничения порядка полей обязательны.

[Полное руководство](guide.ru.md) · [Архитектура](architecture.ru.md)
· [Проверка входа](safety.ru.md) · [Примеры](../examples/README.md)
· [qsl на crates.io](https://crates.io/crates/qsl).

[Переход на 0.2.0 и удалённая feature native](migration-0.2.0.ru.md).
