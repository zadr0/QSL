# QSL 0.2.0 attribute and configuration reference

[aggressive](aggressive.md) · [alignment](alignment.md) · [binary_opt](binary_opt.md) · [bitmap](bitmap.md) · [cache](cache.md) · [compact](compact.md) · [decompress](decompress.md) · [default](default.md) · [inline](inline.md) · [lifetime](lifetime.md) · [loyal_cache](loyal_cache.md) · [macros](macros.md) · [opt_table](opt_table.md) · [predict_seq](predict_seq.md) · [qps](qps.md) · [quant](quant.md) · [rt](rt.md) · [safety](safety.md) · [slots](slots.md) · [state_bits](state_bits.md) · [transmutate](transmutate.md)

All mechanisms are opt-in; required fields must precede optional fields. QPS packs operands, PS uses length, OT selects readers, BM controls mask traversal. Native-layout APIs do not require a Windows Cargo feature. [Complete guide](../../guide.en.md).
