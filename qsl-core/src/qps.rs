//! Lossless QPS operand codecs, independent of field-presence prediction.
//!
//! ByteQps groups any serialized payload into exact radix operands and restores
//! its original bytes. The original byte length is stored without quantization.
//! A separate scalar specialization packs statically bounded numeric ranks;
//! quotient and remainder are retained. Q2/Q4/Q8/Q16 choose generated kernels.
//! `opt_table` owns length-based prediction. `quant=Qn` aliases the scalar
//! schema selection; the outer byte-envelope API is always explicit.
pub use crate::payload_quant::{
    PayloadQuantPlan as QpsPlan, QuantField as QpsField, QuantizedFrame, QuantizedLease,
    QuantizedOut, QuantizedRecord, QuantizedRegions,
};
/// Revision of the specialized scalar representation; the byte envelope has its own revision.
/// Peers must agree on schema/configuration; no runtime format guessing occurs.
pub const QPS_CODEC_REVISION: u32 = 2;
/// General whole-payload operands, independent of the scalar specialization.
pub use crate::qps_bytes::{
    ByteFrame, ByteMode, ByteQps, CheckedBorrowedRecord, CheckedRecord, BYTE_FRAME_HEADER,
    BYTE_QPS_CODEC_REVISION,
};
