# Сравнение протоколов / Protocol comparison

Получено проверенных случаев / validated cases: 16 / 100.

Условия: обычная фоновая нагрузка Windows; порог простоя отключён пользователем. Conditions: ordinary Windows background load; idle gate waived by user. Noise is annotated, not used to block execution.

Не прошедшие качество данные сохраняются отдельно и не используются для вывода о быстродействии. Rejected observations are retained and excluded from performance conclusions.

[Интерактивные графики / charts](report.html) · [Методика / method](METHOD.md) · [Кандидаты оптимизации](OPTIMIZATION_CANDIDATES.ru.md) · [Безопасность](SECURITY.ru.md)

| Case | T / record, ns | 99% CI | P, B/ns | Peak commit, MiB |
|---|---:|---:|---:|---:|
| codec/mixed/read/b16/qsl_default | 1986.336 | 1971.564–2002.916 | 0.9487 | 14.32 |
| codec/mixed/read/b16/qsl_optimized | 2046.666 | 2000.908–2112.715 | 0.9208 | 12.55 |
| codec/mixed/read/b1/protobuf | 2264.805 | 2226.532–2308.876 | 0.8321 | 14.25 |
| codec/mixed/read/b4/qsl_default | 1972.104 | 1957.635–1992.456 | 0.9556 | 14.25 |
| codec/mixed/read/b64/flatbuffers | 5250363.515 | 2332.701–20994220.391 | 0.0004 | 14.35 |
| codec/mixed/read/b64/qsl_optimized | 1965.515 | 1955.176–1976.769 | 0.9588 | 13.73 |
| codec/mixed/write/b4/qsl_default | 72.159 | 71.191–73.295 | 26.1161 | 14.30 |
| codec/numeric/read/b1/bincode | 27.740 | 27.326–28.381 | 2.6496 | 8.02 |
| codec/numeric/read/b1/capnp | 73.545 | 72.783–74.405 | 0.9994 | 7.84 |
| codec/numeric/read/b1/flatbuffers | 72.225 | 71.567–72.965 | 1.0176 | 7.97 |
| codec/numeric/read/b1/qsl_default | 125.519 | 124.577–126.577 | 0.5856 | 7.39 |
| transport/mixed/request_ack/b16/grpc_stream | 11876.572 | 11525.004–12284.116 | 0.1587 | 15.22 |
| transport/mixed/request_ack/b1/qsl_optimized_tcp | 53596.672 | 51754.659–56219.827 | 0.0352 | 12.52 |
| transport/mixed/request_ack/b4/protobuf_tcp | 15556.472 | 15432.983–15726.568 | 0.1211 | 11.39 |
| transport/mixed/request_ack/b64/bincode_tcp | 3604.260 | 3559.947–3658.078 | 0.5229 | 12.31 |
| transport/mixed/request_ack/b64/qsl_default_tcp | 4069.048 | 3937.622–4233.905 | 0.4631 | 12.38 |
