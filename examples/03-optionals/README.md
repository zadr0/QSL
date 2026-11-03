# Опциональные поля / Optional fields

Самостоятельный `[opt_table]`: тег → статическая таблица → читатель. Он сохраняет байты default. `predict_seq` использует длину и числовой индекс; `[predict_seq,opt_table]` сохраняет прежний смешанный формат. Опциональные поля всегда идут в конце.

Standalone `[opt_table]`: tag → static table → reader. It preserves default bytes. `predict_seq` uses length and a numeric index; `[predict_seq,opt_table]` preserves the former mixed format. Optional fields must be last.

```powershell
cargo run --manifest-path examples/Cargo.toml --example optionals
```

Программа проверяет все состояния N=3. Сравнение default / opt_table использует одинаковые кадры, а checked-чтение включает проверку всего кадра. Старые измерения в PERFORMANCE.md относятся к прежней смешанной реализации; новый контроль опубликован отдельно.

The program checks every N=3 state. Default / opt_table compare identical frames; checked decoding includes full-frame validation. Older PERFORMANCE.md measurements belong to the former mixed implementation; the new control is published separately.
