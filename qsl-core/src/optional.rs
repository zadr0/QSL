//! Validated, allocation-free access to an opt_table directory.
//! Wire: count:u8, count little-endian u32 offsets, optional payload.
//! Zero means absent; a present value stores its payload-relative offset plus one.
//! The directory follows all required fields. Appending optional fields preserves
//! existing indices. Changing required fields or optional types needs a new schema.
use crate::{QslError, QslResult};
#[derive(Clone, Copy, Debug)]
pub struct OptionalOffsets<'a> {
    entries: &'a [u8],
    payload: &'a [u8],
}
impl<'a> OptionalOffsets<'a> {
    /// Check metadata once. This does not validate individual field codecs/UTF-8.
    pub fn parse(table_and_payload: &'a [u8]) -> QslResult<Self> {
        let count = *table_and_payload.first().ok_or(QslError::InvalidSchema)? as usize;
        if !(2..=64).contains(&count) {
            return Err(QslError::InvalidSchema);
        }
        let end = 1 + count * 4;
        if table_and_payload.len() < end {
            return Err(QslError::InvalidSchema);
        }
        let value = Self {
            entries: &table_and_payload[1..end],
            payload: &table_and_payload[end..],
        };
        let mut previous = None;
        for i in 0..count {
            if let Some(offset) = value.offset(i) {
                if offset > value.payload.len()
                    || previous.is_some_and(|p| offset < p)
                    || (previous.is_none() && offset != 0)
                {
                    return Err(QslError::InvalidSchema);
                }
                previous = Some(offset);
            }
        }
        if previous.is_none() && !value.payload.is_empty() {
            return Err(QslError::InvalidSchema);
        }
        Ok(value)
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.entries.len() >> 2
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    /// O(1) index lookup; indices are compiler-assigned optional ordinals.
    #[inline(always)]
    pub fn offset(&self, index: usize) -> Option<usize> {
        if index >= self.len() {
            return None;
        }
        let start = index * 4;
        let raw = u32::from_le_bytes(self.entries[start..start + 4].try_into().unwrap());
        if raw == 0 {
            None
        } else {
            Some(raw as usize - 1)
        }
    }
    /// Remaining payload at a field's start; the field codec determines its end.
    #[inline(always)]
    pub fn field_tail(&self, index: usize) -> Option<&'a [u8]> {
        self.offset(index).map(|offset| &self.payload[offset..])
    }
}
