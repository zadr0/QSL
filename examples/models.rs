#[qsl::qsl_macro]
pub struct Header { pub id: u32 }
#[qsl::schem_macro]
pub struct Trace { pub header: Header, pub kind: u8 }
