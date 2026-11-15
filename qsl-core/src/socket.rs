//! Allocator-independent binary framing over an application transport.
use crate::{BorrowSerializable, QslError, QslResult, Serializable};
/// Platform I/O is supplied by the application; codecs never depend on std::net.
pub trait ByteTransport {
    type Error;
    fn read(&mut self, out: &mut [u8]) -> Result<usize, Self::Error>;
    fn write(&mut self, input: &[u8]) -> Result<usize, Self::Error>;
    fn abort(&mut self);
    fn interrupted(_error: &Self::Error) -> bool {
        false
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum SocketError<E> {
    Transport(E),
    Protocol(QslError),
    UnexpectedEof,
    WriteZero,
    Poisoned,
}
impl<E: core::fmt::Display> core::fmt::Display for SocketError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transport(e) => e.fmt(f),
            Self::Protocol(e) => e.fmt(f),
            Self::UnexpectedEof => f.write_str("incomplete QSL frame"),
            Self::WriteZero => f.write_str("transport accepted no bytes"),
            Self::Poisoned => f.write_str("QSL transport framing is poisoned"),
        }
    }
}
impl<E: core::error::Error + 'static> core::error::Error for SocketError<E> {}
pub struct RawSocket<S: ByteTransport> {
    stream: S,
    max_frame: usize,
    poisoned: bool,
}
impl<S: ByteTransport> RawSocket<S> {
    pub fn new(stream: S, max_frame: usize) -> QslResult<Self> {
        if max_frame == 0 || max_frame > u32::MAX as usize {
            return Err(QslError::InvalidValue);
        }
        Ok(Self {
            stream,
            max_frame,
            poisoned: false,
        })
    }
    fn poison(&mut self) {
        self.poisoned = true;
        self.stream.abort()
    }
    fn read_exact(&mut self, mut out: &mut [u8]) -> Result<(), SocketError<S::Error>> {
        if self.poisoned {
            return Err(SocketError::Poisoned);
        }
        while !out.is_empty() {
            match self.stream.read(out) {
                Ok(0) => {
                    self.poison();
                    return Err(SocketError::UnexpectedEof);
                }
                Ok(n) if n <= out.len() => out = &mut out[n..],
                Ok(_) => {
                    self.poison();
                    return Err(SocketError::Protocol(QslError::InvalidValue));
                }
                Err(e) if S::interrupted(&e) => {}
                Err(e) => {
                    self.poison();
                    return Err(SocketError::Transport(e));
                }
            }
        }
        Ok(())
    }
    fn write_all(&mut self, mut input: &[u8]) -> Result<(), SocketError<S::Error>> {
        if self.poisoned {
            return Err(SocketError::Poisoned);
        }
        while !input.is_empty() {
            match self.stream.write(input) {
                Ok(0) => {
                    self.poison();
                    return Err(SocketError::WriteZero);
                }
                Ok(n) if n <= input.len() => input = &input[n..],
                Ok(_) => {
                    self.poison();
                    return Err(SocketError::Protocol(QslError::InvalidValue));
                }
                Err(e) if S::interrupted(&e) => {}
                Err(e) => {
                    self.poison();
                    return Err(SocketError::Transport(e));
                }
            }
        }
        Ok(())
    }
    pub fn receive<'a>(
        &mut self,
        storage: &'a mut [u8],
    ) -> Result<&'a [u8], SocketError<S::Error>> {
        let mut prefix = [0; 4];
        self.read_exact(&mut prefix)?;
        let len = u32::from_le_bytes(prefix) as usize;
        if len > self.max_frame || len > storage.len() {
            self.poison();
            return Err(SocketError::Protocol(QslError::BufferTooSmall));
        }
        self.read_exact(&mut storage[..len])?;
        Ok(&storage[..len])
    }
    pub fn send_bytes(&mut self, bytes: &[u8]) -> Result<(), SocketError<S::Error>> {
        if bytes.len() > self.max_frame {
            return Err(SocketError::Protocol(QslError::SizeLimitExceeded));
        }
        self.write_all(&(bytes.len() as u32).to_le_bytes())?;
        self.write_all(bytes)
    }
    fn send_encoded(
        &mut self,
        capacity: usize,
        storage: &mut [u8],
        encode: impl FnOnce(*mut u8) -> usize,
    ) -> Result<usize, SocketError<S::Error>> {
        if self.poisoned {
            return Err(SocketError::Poisoned);
        }
        let needed = capacity
            .checked_add(4)
            .ok_or(SocketError::Protocol(QslError::LengthOverflow))?;
        if needed > storage.len() || capacity > self.max_frame {
            return Err(SocketError::Protocol(QslError::BufferTooSmall));
        }
        let used = encode(storage[4..].as_mut_ptr());
        if used > capacity {
            return Err(SocketError::Protocol(QslError::InvalidValue));
        }
        storage[..4].copy_from_slice(&(used as u32).to_le_bytes());
        self.write_all(&storage[..4 + used])?;
        Ok(used)
    }
    /// # Safety
    /// Values satisfy the exact schema and serializer size/nonoverlap contracts.
    pub unsafe fn send_record<T: Serializable>(
        &mut self,
        value: &T,
        storage: &mut [u8],
    ) -> Result<usize, SocketError<S::Error>> {
        self.send_encoded(value.serialized_size(), storage, |p| {
            value.serialize_into_unsafe(p)
        })
    }
    /// # Safety
    /// Values satisfy the borrowed serializer's size, lifetime and nonoverlap contracts.
    pub unsafe fn send_borrowed_record<'a, T: BorrowSerializable<'a>>(
        &mut self,
        value: &T,
        storage: &mut [u8],
    ) -> Result<usize, SocketError<S::Error>> {
        self.send_encoded(value.serialized_size(), storage, |p| {
            value.serialize_into_unsafe(p)
        })
    }
    pub fn transport(&self) -> &S {
        &self.stream
    }
    pub fn transport_mut(&mut self) -> &mut S {
        &mut self.stream
    }
    pub fn into_transport(self) -> S {
        self.stream
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Partial {
        input: &'static [u8],
        at: usize,
        output: [u8; 32],
        written: usize,
        closed: bool,
    }
    impl Partial {
        fn new(input: &'static [u8]) -> Self {
            Self {
                input,
                at: 0,
                output: [0; 32],
                written: 0,
                closed: false,
            }
        }
    }
    impl ByteTransport for Partial {
        type Error = core::convert::Infallible;
        fn read(&mut self, out: &mut [u8]) -> Result<usize, Self::Error> {
            let n = out.len().min(1).min(self.input.len() - self.at);
            out[..n].copy_from_slice(&self.input[self.at..self.at + n]);
            self.at += n;
            Ok(n)
        }
        fn write(&mut self, b: &[u8]) -> Result<usize, Self::Error> {
            let n = b.len().min(1).min(32 - self.written);
            self.output[self.written..self.written + n].copy_from_slice(&b[..n]);
            self.written += n;
            Ok(n)
        }
        fn abort(&mut self) {
            self.closed = true;
        }
    }
    #[test]
    fn partial_operations_keep_boundaries() {
        let mut s =
            RawSocket::new(Partial::new(&[3, 0, 0, 0, 1, 2, 3, 1, 0, 0, 0, 42]), 8).unwrap();
        let mut b = [0; 8];
        assert_eq!(s.receive(&mut b).unwrap(), [1, 2, 3]);
        assert_eq!(s.receive(&mut b).unwrap(), [42]);
        s.send_bytes(&[4, 5]).unwrap();
        assert_eq!(&s.transport().output[..6], &[2, 0, 0, 0, 4, 5]);
    }
    #[test]
    fn oversized_or_truncated_frames_abort_and_poison() {
        for bytes in [&[9, 0, 0, 0][..], &[3, 0, 0, 0, 1][..]] {
            let mut s = RawSocket::new(Partial::new(bytes), 8).unwrap();
            let mut b = [0; 8];
            assert!(s.receive(&mut b).is_err());
            assert!(s.transport().closed);
            assert_eq!(s.receive(&mut b), Err(SocketError::Poisoned));
        }
    }
}
