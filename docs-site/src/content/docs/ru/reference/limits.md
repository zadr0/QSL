---
title: "Ограничения и исходники"
description: "Документация QSL: Ограничения и исходники."
qslResources: [{"label": "Репозиторий QSL", "url": "https://github.com/zadr0/QSL"}, {"label": "Расширение", "url": "https://github.com/zadr0/QSL-Extension"}]
---
Механизм/сочетание | Состояние
---|---
PS, OT, PS+OT, BM | Отдельные действующие механики; совместимость проверяется
Physical auto + pinned wire | Реализовано для доказуемых layouts
Private bitmap + default/BM/OT | Реализовано; opaque/QPS/PS ограничения сохраняются
COMPACT + QPS | Отклоняется
Dyn line + raw | Реализовано
Dyn index32/64 + raw/выбранный COMPACT | Реализовано
Dyn + QPS/PS/OT/BM/LZ/RT/overlay/private bitmap | Отклоняется текущим backend
Index16, line+COMPACT | Не определено/не поддержано
Вложенные массивы | Поддержаны до 32 уровней текстового синтаксиса; длины, element-storage и бюджеты проверяются рекурсивно
Regional IR и initialized-byte map | Реализованы; модель операций не является доказательством оптимальности
CPU cost model, GPR/BMI2/SIMD kernels | Приближённый planner и reference kernels; включена только измеренная PEXT-запись 4×12
Auto-wire | Реализован явный v1; несовместимые сочетания отклоняются
len+1 string storage | Реализовано через явный shift=true, wire остаётся прежним
LLVM backend, QBProto handshake | Пока планы
. Распознанная опция, доказанная формула или успешный частный тест не являются готовой универсальной оптимизацией.

### Термины
Термин | Значение
---|---
Схема | Контракт полей, значений, присутствия и кодеков
Wire / payload | Реальные биты сообщения / полезная область выбранного формата
Self / layout | Физическое Rust-представление
Rank | Обратимый номер допустимого значения
Byte barrier | Завершение частичного байта перед byte-представлением
Cursor / window | Позиция I/O / доказанный диапазон операций
Padding | Выравнивание wire либо промежутки Self; категории различаются
Scratch | Caller-owned временный буфер
Lease | Исключительное владение слотом до Drop
Zero-copy | Borrow живого storage с layout/валидностью/lifetime
### Основные файлы
Область | Исходники
---|---
Schema / validation | [types.rs](/source/qsl-lexer/src/types.rs), [validation.rs](/source/qsl-core/src/validation.rs)
Domains / hashes | [domain.rs](/source/qsl-core/src/domain.rs), [schema_hash.rs](/source/qsl-core/src/schema_hash.rs)
Windows / I/O | [analysis.rs](/source/qsl-lexer/src/analysis.rs), [bit_io.rs](/source/qsl-core/src/bit_io.rs)
Physical / storage | [layout.rs](/source/qsl-lexer/src/layout.rs), [optimizer.rs](/source/qsl-core/src/optimizer.rs), [storage.rs](/source/qsl-core/src/storage.rs)
Frontend | [lexer.rs](/source/qsl-lexer/src/lexer.rs), [parser.rs](/source/qsl-lexer/src/parser.rs), [builder.rs](/source/qsl-lexer/src/builder.rs)
Backend | [frontend.rs](/source/qsl-interpreter/src/frontend.rs), [interpreter.rs](/source/qsl-interpreter/src/interpreter.rs), [backend.rs](/source/qsl-interpreter/src/backend.rs)
Specific codegen | [numeric](/source/qsl-interpreter/src/codegen_numeric.rs), [checked](/source/qsl-interpreter/src/codegen_checked.rs), [grouped](/source/qsl-interpreter/src/codegen_grouped.rs), [dyn](/source/qsl-interpreter/src/codegen_dynamic.rs)
Runtime | [serialize.rs](/source/qsl-core/src/serialize.rs), [rt.rs](/source/qsl-rt/src/lib.rs), [socket.rs](/source/qsl-core/src/socket.rs)

 ·  · .

Репозиторий QSL · Расширение. Ссылки не означают, что локальные изменения уже опубликованы.
