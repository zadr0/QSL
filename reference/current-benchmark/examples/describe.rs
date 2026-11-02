use qsl::Serializable;
use qsl_current_bench::{Qps, Raw};
fn main() {
    let raw: Vec<_> = (0..16)
        .map(|m| Raw {
            base: -7,
            a: 13,
            b: (m & 1 != 0).then_some(999),
            c: (m & 2 != 0).then_some(-37),
            constant: (m & 4 != 0).then_some(7),
            wide: (m & 8 != 0).then_some(0x123456789abcdef0),
        })
        .collect();
    let packed: Vec<_> = raw
        .iter()
        .map(|v| Qps {
            base: v.base,
            a: v.a,
            b: v.b,
            c: v.c,
            constant: v.constant,
            wide: v.wide,
        })
        .collect();
    let a: Vec<_> = raw.iter().map(|v| v.serialized_size()).collect();
    let b: Vec<_> = packed.iter().map(|v| v.serialized_size()).collect();
    println!(
        "{{\"rawFrameBytes\":{a:?},\"qpsFrameBytes\":{b:?},\"rawNativeBytes\":{},\"qpsNativeBytes\":{},\"privatePaddingBytes\":{},\"rawHeaderBytes\":{}}}",
        std::mem::size_of::<Raw>(),
        std::mem::size_of::<Qps>(),
        Qps::QUANT_PADDING_BYTES,
        Raw::OPT_TABLE_HEADER_BYTES
    );
}
