use qsl_examples::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for mask in 0u8..8 {
        let a = Optional3Default {
            id: 42,
            f0: (mask & 1 != 0).then_some(11),
            f1: (mask & 2 != 0).then_some(22),
            f2: (mask & 4 != 0).then_some(33),
        };
        let b = Optional3Table {
            id: a.id,
            f0: a.f0,
            f1: a.f1,
            f2: a.f2,
        };
        let raw = a.serialize_checked()?;
        let table = b.serialize_checked()?;
        let decoded = Optional3Table::deserialize_checked(&table)?;
        assert_eq!((decoded.f0, decoded.f1, decoded.f2), (a.f0, a.f1, a.f2));
        println!(
            "mask={mask:03b}: default={} B, opt_table={} B; bytes={table:02x?}",
            raw.len(),
            table.len()
        );
    }
    Ok(())
}
