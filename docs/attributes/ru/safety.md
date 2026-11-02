# safety

[Текущая архитектура и полный контракт](../../architecture.ru.md).

Генерирует validate, validate_values, deserialize_checked и checked-чтение в конечную область. Проверяет кадр до trusted-декодирования. Не включает неявные проверки в каждый unsafe-вызов; custom-кодек требует собственного валидатора.
