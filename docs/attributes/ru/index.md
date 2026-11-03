# Атрибуты схемы

[qps](qps.md) · [quant](quant.md) · [state_bits](state_bits.md) · [opt_table](opt_table.md) · [safety](safety.md) · [inline](inline.md) · [rt](rt.md) · [macros](macros.md) · [predict_seq](predict_seq.md) · [binary_opt](binary_opt.md) · [decompress](decompress.md) · [lifetime](lifetime.md) · [compact](compact.md) · [bitmap](bitmap.md) · [alignment](alignment.md) · [default](default.md)

[Актуальная архитектура](../../architecture.ru.md). QPS сжимает числовым операндом; predict_seq предсказывает по длине; opt_table выбирает читателя по тегу. `[qps]` по умолчанию использует Q16. Старое кодирование D/индексов — `state_bits`; `quant=Qn` — синоним QPS.
