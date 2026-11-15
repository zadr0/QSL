# Проверки и контроль ресурсов

Профиль выполняет проверяемое чтение всех значений. Подготовленные буферы и прогрев не отменяют валидацию. Во всех кодеках действуют одинаковые ограничения значений: восемь чисел 0..4095; UTF-8 1..4096 B; массив 0..256 B; два optional-поля. Сужение исходных u64 проверяется до приведения типов. Целый codec-кадр ограничен 65535 B. Транспортный batch имеет отдельные границы: не более 64 записей и 256 KiB.

| Реализация | Проверка входа | Что задаёт разработчик |
|---|---|---|
| QSL | Полный ValidationCursor, диапазоны, UTF-8, теги/длина, общий бюджет до trusted reader | MAX_DATA_SIZE, диапазоны, режим владения и буфер; prepared QPS требует инициализированного private padding |
| Cap’n Proto | Безаллокационный reader, проверяемые accessor, полный обход, traversal/depth limits | ReaderOptions: 8192 слов и глубина 16; прикладные диапазоны и длины |
| FlatBuffers | Проверяемый root/verifier до accessor, затем прикладные диапазоны | max_depth=16, max_tables=16, max_apparent_size=65535; обязательная Values-структура, строка и массив |
| Protobuf/prost | Предел кадра до decode/merge; UTF-8 SDK; ровно восемь значений и диапазоны после decode | Размер кадра, форма repeated-поля, приватный reuse scratch; результат публикуется только при Ok |
| Bincode 2 | Ограниченный borrow_decode, fixed-int little-endian, проверка потреблённых байтов | Типы и конфигурация обеих сторон; лимит 65535; ограничения значений после decode |
| gRPC/tonic | send/receive caps 256 KiB, типизированный Protobuf, прикладная проверка каждой записи | Лимиты сообщений, concurrency/window/backpressure; unary или bidirectional stream |

У Protobuf/gRPC прикладные ограничения repeated-полей проверяются после создания соответствующих контейнеров. Предел wire-размера не равен пределу heap-памяти: маленькие повторяющиеся сообщения могут давать больший объём объектов. Лимит JobObject 8 GiB относится ко всему измеряемому процессу и его потомкам. Это отдельный ресурсный предел, а не доказательство, что любая библиотека всегда вернёт Result при системном OOM.

Проверки корпуса включают повреждение байтов, обрезанные кадры, чрезмерный размер, неверные диапазоны, сужение без усечения, соответствие prepared/exact QSL и owned/reuse Protobuf, а также реальные TCP, unary gRPC и stream gRPC. Успешный прогон конечного корпуса не является формальным доказательством отсутствия всех дефектов SDK или unsafe-кода. Хэш потребителя в стенде служит контролю работы; он не обеспечивает аутентификацию. Встроенного согласования схем QBProto здесь нет. TCP-handshake стенда различает его версию и кодек; схемы зафиксированы исходниками.

Все соединения — loopback, persistent, без TLS и транспортного сжатия. Поэтому результат не является оценкой защищённого удалённого production-развёртывания. TCP — явный прикладной framed-протокол; Cap’n Proto RPC им не подменяется. Ошибки чтения/фрейминга закрывают TCP-соединение. Тонкости лимитов и публикации частично изменённого Prost scratch должен учитывать код приложения.

Источники SDK: [Cap’n Proto ReaderOptions](https://docs.rs/capnp/0.27.3/capnp/message/struct.ReaderOptions.html), [FlatBuffers Rust](https://flatbuffers.dev/languages/rust/), [Prost Message](https://docs.rs/prost/0.14.4/prost/trait.Message.html), [Tonic message caps](https://docs.rs/tonic/0.14.9/tonic/server/struct.Grpc.html), [Bincode 2 configuration](https://docs.rs/bincode/2.0.1/bincode/config/struct.Configuration.html).
