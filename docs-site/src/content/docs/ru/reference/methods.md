---
title: "Методы сериализации и чтения"
description: "Документация QSL: Методы сериализации и чтения."
---

Набор методов зависит от схемы. У dyn, lifetime, QPS и native есть собственные сигнатуры. Сгенерированный Rust — точный API выбранного варианта.
Метод | Контракт
---|---
validate_values() | Значения Self и их суммарный бюджет
validate(input) | Проверка input там, где генерируется checked API
serialize_checked() | Проверяемый fallible-выход: `StackOut<N>` либо `Vec<u8>`, выбранный по максимуму схемы.
serialize_checked_to(&mut [u8]) | Проверка значений/capacity, число записанных wire-байтов
deserialize_checked(input) | Checked-материализация при safety
deserialize_checked_into(input,&mut MaybeUninit&lt;Self&gt;) | New-destination; dyn дополнительно принимает scratch
serialized_size() | Вместимость; для сжатых строк возможен raw upper bound
serialize_into_unsafe(ptr) | Вместимость и валидность обеспечивает вызывающий код
deserialize_from_raw_parts(ptr,len) | Trusted length-aware reader обычного record
deserialize_into / deserialize_reuse / deserialize_prepared_into | Unsafe назначение и подготовленное reuse по generated-контракту
comp_size / comp_write / comp_read | Совместимые COMPACT-обёртки, если сгенерированы
Передавать далее нужно только buffer[..used], а не незаписанную capacity или private QPS padding.

Подготовленный unsafe reuse требует достаточной ёмкости до вызова и сохраняет пригодные буферы String, Vec и boxed InlineString. Их рост в этом пути не выполняется. Инициализированный mutable-объект и неинициализированное назначение имеют разные контракты.

Trusted-методы Serializable остаются unsafe. Вызывающий код обеспечивает допустимые значения, полный вход, достаточную память, выравнивание и времена жизни.

Для обработки ошибок используйте serialize_checked. Совместимые методы не подменяют checked-контракт и отдельный unsafe-контракт.

Checked cursor проверяет доступность, арифметические переполнения, домены/spare-коды, UTF-8, присутствие, padding, восстановление и бюджет до публикации результата. Агрегированный check разрешён только для действительно доступного острова. Он не разрешает load через absent optional, padding Self или конец кадра.

Unsafe-контракт включает полный валидный кадр нужной схемы, capacity/инициализацию, выравнивание назначения, отсутствие запрещённого перекрытия и корректный lifetime. У prepared QPS дополнительно учитывается private storage.
Ошибка | Смысл
---|---
UnexpectedEof | Недостаточно input
BufferTooSmall | Недостаточно назначения/scratch
LengthOverflow | Переполнение длины/offset
InvalidWidth / InvalidSchema | Неверная ширина/несовместимость
InvalidValue | Домен, spare-код или структура значений
InvalidUtf8 / InvalidPadding | Некорректный текст/padding
SizeLimitExceeded | Лимит кадра, восстановления или значений
AllocationFailed | Ошибка fallible allocation
PoolExhausted | Нет слота при try_acquire
Пользовательский QslCustom обязан соблюдать собственный валидатор/кодек. Общая проверка QSL не доказывает корректность произвольной пользовательской реализации. Ошибка записи не является обещанием транзакционного отката буфера; при ошибке его содержимое не публикуется как готовый кадр.

Генератор выбирает тип выхода по доказанному максимуму кадра. Если он не превышает STACK_LIMIT, включая равенство, обычная сериализация возвращает `StackOut<N>`; больший или неизвестный максимум выбирает alloc::`Vec<u8>`. Передаваемые заголовки входят в порог; служебная длина и выравнивание самого выходного объекта — нет.
