//! Windows native storage and blocking TCP, enabled by `native`.
//! Regions keep allocation and page faults outside the repeated codec path.
//! Shared regions use offsets and immutable publication, including across processes.
mod memory;
mod socket;
pub use memory::{
    NativeRegion, RegionBuffer, RegionSlot, RegionValue, SharedBytes, SharedRegion, SharedWrite,
};
pub use socket::{NativeListener, NativeSocket};
