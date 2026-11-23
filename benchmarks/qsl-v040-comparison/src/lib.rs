#![allow(clippy::type_complexity)]
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/qsl_gen.rs"));
}
pub mod records_capnp {
    include!(concat!(env!("OUT_DIR"), "/records_capnp.rs"));
}
pub mod codecs;
pub mod fixtures;
pub use codecs::*;
pub use fixtures::*;
pub mod cases;
