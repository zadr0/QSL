//! Direct binary TCP framing. A frame is u32 little-endian length + QSL bytes.
//! Transport framing is independent of schema optional metadata. Callers reuse
//! buffers; this adapter neither allocates a message buffer nor changes OS tuning.
use crate::{BorrowSerializable, Serializable};
use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

pub struct RawSocket {
    stream: TcpStream,
    max_frame: usize,
}
impl RawSocket {
    pub fn new(stream: TcpStream, max_frame: usize) -> io::Result<Self> {
        if max_frame == 0 || max_frame > u32::MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid maximum frame",
            ));
        }
        stream.set_nodelay(true)?;
        Ok(Self { stream, max_frame })
    }
    fn read_exact(&mut self, bytes: &mut [u8]) -> io::Result<()> {
        if let Err(error) = self.stream.read_exact(bytes) {
            let _ = self.stream.shutdown(std::net::Shutdown::Both);
            return Err(error);
        }
        Ok(())
    }
    /// Read exactly one bounded frame into caller-owned storage.
    /// An oversized frame terminates this stream to avoid losing framing state.
    pub fn receive<'a>(&mut self, storage: &'a mut [u8]) -> io::Result<&'a [u8]> {
        let mut prefix = [0u8; 4];
        self.read_exact(&mut prefix)?;
        let len = u32::from_le_bytes(prefix) as usize;
        if len > self.max_frame || len > storage.len() {
            let _ = self.stream.shutdown(std::net::Shutdown::Both);
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "frame exceeds capacity",
            ));
        }
        self.read_exact(&mut storage[..len])?;
        Ok(&storage[..len])
    }
    fn send_encoded(
        &mut self,
        capacity: usize,
        storage: &mut [u8],
        encode: impl FnOnce(*mut u8) -> usize,
    ) -> io::Result<usize> {
        let needed = capacity
            .checked_add(4)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "frame size overflow"))?;
        if needed > storage.len() || capacity > self.max_frame {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "insufficient frame capacity",
            ));
        }
        let used = encode(storage[4..].as_mut_ptr());
        storage[..4].copy_from_slice(&(used as u32).to_le_bytes());
        if let Err(error) = self.stream.write_all(&storage[..4 + used]) {
            let _ = self.stream.shutdown(std::net::Shutdown::Both);
            return Err(error);
        }
        Ok(used)
    }
    /// Serialize directly after the transport prefix and send the initialized bytes.
    /// # Safety
    /// Every record field must satisfy its schema's range and UTF-8 contracts.
    pub unsafe fn send_record<T: Serializable>(
        &mut self,
        value: &T,
        storage: &mut [u8],
    ) -> io::Result<usize> {
        self.send_encoded(value.serialized_size(), storage, |p| {
            value.serialize_into_unsafe(p)
        })
    }
    /// # Safety
    /// Every record field must satisfy its schema's range and UTF-8 contracts.
    pub unsafe fn send_borrowed_record<'a, T: BorrowSerializable<'a>>(
        &mut self,
        value: &T,
        storage: &mut [u8],
    ) -> io::Result<usize> {
        self.send_encoded(value.serialized_size(), storage, |p| {
            value.serialize_into_unsafe(p)
        })
    }
    pub fn stream(&self) -> &TcpStream {
        &self.stream
    }
}
