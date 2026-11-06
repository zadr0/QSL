pub type QslResult<T> = Result<T, QslError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QslError {
    UnexpectedEof,
    BufferTooSmall,
    LengthOverflow,
    InvalidWidth,
    InvalidValue,
    InvalidUtf8,
    InvalidSchema,
    InvalidPadding,
    PoolExhausted,
    SizeLimitExceeded,
    AllocationFailed,
}

impl std::fmt::Display for QslError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QSL error: {self:?}")
    }
}
impl std::error::Error for QslError {}

/// Exact output length for interleaved bit and byte writes.
#[derive(Debug, Default, Clone, Copy)]
pub struct QslSizer {
    bytes: usize,
    bits: usize,
}
impl QslSizer {
    /// Add `n` bits with checked overflow arithmetic, folding completed bytes.
    /// This utility is for code generation, not serialized object I/O.
    pub fn bits(&mut self, n: usize) -> QslResult<()> {
        let total = self.bits.checked_add(n).ok_or(QslError::LengthOverflow)?;
        self.bytes = self
            .bytes
            .checked_add(total / 8)
            .ok_or(QslError::LengthOverflow)?;
        self.bits = total % 8;
        Ok(())
    }
    /// Round the pending bits up to the next byte boundary, rejecting overflow.
    pub fn align(&mut self) -> QslResult<()> {
        if self.bits != 0 {
            self.bytes = self.bytes.checked_add(1).ok_or(QslError::LengthOverflow)?;
            self.bits = 0;
        }
        Ok(())
    }
    /// Append `n` whole bytes after byte alignment, rejecting overflow.
    pub fn bytes(&mut self, n: usize) -> QslResult<()> {
        self.align()?;
        self.bytes = self.bytes.checked_add(n).ok_or(QslError::LengthOverflow)?;
        Ok(())
    }
    /// Return the final whole-byte size, including the last partial byte.
    pub fn finish(mut self) -> QslResult<usize> {
        self.align()?;
        Ok(self.bytes)
    }
}
