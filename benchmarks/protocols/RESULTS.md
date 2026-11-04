# Сравнение протоколов / Protocol comparison

Получено проверенных случаев / validated cases: 2 / 100.

Условия: обычная фоновая нагрузка Windows; порог простоя отключён пользователем. Conditions: ordinary Windows background load; idle gate waived by user. Noise is annotated, not used to block execution.

Не прошедшие качество данные сохраняются отдельно и не используются для вывода о быстродействии. Rejected observations are retained and excluded from performance conclusions.

[Интерактивные графики / charts](report.html) · [Методика / method](METHOD.md) · [Кандидаты оптимизации](OPTIMIZATION_CANDIDATES.ru.md) · [Безопасность](SECURITY.ru.md)

| Case | T / record, ns | 99% CI | P, B/ns | Peak commit, MiB |
|---|---:|---:|---:|---:|
| codec/mixed/read/b16/qsl_default | 1986.336 | 1971.564–2002.916 | 0.9487 | 14.32 |
| codec/mixed/read/b16/qsl_optimized | 2046.666 | 2000.908–2112.715 | 0.9208 | 12.55 |
