//! Wire-policy constants shared by analysis and runtime.
/// Raw-copy threshold for QSL-LZ.
pub const STRING_COMPRESSION_MIN_BYTES: usize = 32;
/// Encoder policy revision; the decoder format remains QSL-LZ v1.
pub const STRING_ENCODER_REVISION: u8 = 2;

/// Width of the QSL-LZ shared length/mode word.
pub const fn inline_string_header_bytes(max: usize) -> usize {
    if max <= 127 {
        1
    } else if max <= 32767 {
        2
    } else {
        4
    }
}
