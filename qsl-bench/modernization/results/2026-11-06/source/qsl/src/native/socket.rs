use super::{RegionBuffer, SharedWrite};
use crate::{BorrowSerializable, Serializable};
use std::{
    io,
    mem::{size_of, MaybeUninit},
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV6},
    sync::OnceLock,
};
use windows_sys::Win32::Networking::WinSock::*;

fn error() -> io::Error {
    io::Error::from_raw_os_error(unsafe { WSAGetLastError() })
}
fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
fn startup() -> io::Result<()> {
    static STARTED: OnceLock<i32> = OnceLock::new();
    let result = *STARTED.get_or_init(|| unsafe {
        let mut data = MaybeUninit::<WSADATA>::uninit();
        WSAStartup(0x0202, data.as_mut_ptr())
    });
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(result))
    }
}
struct Handle(SOCKET);
impl Handle {
    fn new(addr: SocketAddr) -> io::Result<Self> {
        startup()?;
        let s = unsafe {
            socket(
                if addr.is_ipv4() { AF_INET } else { AF_INET6 } as i32,
                SOCK_STREAM,
                IPPROTO_TCP,
            )
        };
        if s == INVALID_SOCKET {
            Err(error())
        } else {
            Ok(Self(s))
        }
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            closesocket(self.0);
        }
    }
}
fn with_addr<T>(addr: SocketAddr, call: impl FnOnce(*const SOCKADDR, i32) -> T) -> T {
    match addr {
        SocketAddr::V4(a) => {
            let native = SOCKADDR_IN {
                sin_family: AF_INET,
                sin_port: a.port().to_be(),
                sin_addr: IN_ADDR {
                    S_un: IN_ADDR_0 {
                        S_addr: u32::from_ne_bytes(a.ip().octets()),
                    },
                },
                ..Default::default()
            };
            call(
                (&native as *const SOCKADDR_IN).cast(),
                size_of::<SOCKADDR_IN>() as i32,
            )
        }
        SocketAddr::V6(a) => {
            let native = SOCKADDR_IN6 {
                sin6_family: AF_INET6,
                sin6_port: a.port().to_be(),
                sin6_flowinfo: a.flowinfo().to_be(),
                sin6_addr: IN6_ADDR {
                    u: IN6_ADDR_0 {
                        Byte: a.ip().octets(),
                    },
                },
                Anonymous: SOCKADDR_IN6_0 {
                    sin6_scope_id: a.scope_id(),
                },
            };
            call(
                (&native as *const SOCKADDR_IN6).cast(),
                size_of::<SOCKADDR_IN6>() as i32,
            )
        }
    }
}
fn local_addr(s: SOCKET) -> io::Result<SocketAddr> {
    let mut addr = SOCKADDR_STORAGE::default();
    let mut len = size_of::<SOCKADDR_STORAGE>() as i32;
    if unsafe { getsockname(s, (&mut addr as *mut SOCKADDR_STORAGE).cast(), &mut len) }
        == SOCKET_ERROR
    {
        return Err(error());
    }
    unsafe {
        match addr.ss_family {
            AF_INET => {
                let a = &*(&addr as *const SOCKADDR_STORAGE).cast::<SOCKADDR_IN>();
                Ok(SocketAddr::new(
                    Ipv4Addr::from(a.sin_addr.S_un.S_addr.to_ne_bytes()).into(),
                    u16::from_be(a.sin_port),
                ))
            }
            AF_INET6 => {
                let a = &*(&addr as *const SOCKADDR_STORAGE).cast::<SOCKADDR_IN6>();
                Ok(SocketAddr::V6(SocketAddrV6::new(
                    Ipv6Addr::from(a.sin6_addr.u.Byte),
                    u16::from_be(a.sin6_port),
                    u32::from_be(a.sin6_flowinfo),
                    a.Anonymous.sin6_scope_id,
                )))
            }
            _ => Err(invalid("unknown address family")),
        }
    }
}
fn validate_limit(max_frame: usize) -> io::Result<()> {
    if max_frame == 0 || max_frame > u32::MAX as usize {
        Err(invalid("invalid maximum frame"))
    } else {
        Ok(())
    }
}

/// Direct blocking WinSock TCP. No async runtime or intermediate message buffer.
/// Raw methods transfer bytes; frame methods add a separate u32 little-endian length.
/// A mutable borrow serializes each complete send/receive operation.
pub struct NativeSocket {
    handle: Handle,
    max_frame: usize,
    poisoned: bool,
}
impl NativeSocket {
    pub fn connect(address: SocketAddr, max_frame: usize) -> io::Result<Self> {
        validate_limit(max_frame)?;
        let handle = Handle::new(address)?;
        if with_addr(address, |a, n| unsafe { connect(handle.0, a, n) }) == SOCKET_ERROR {
            return Err(error());
        }
        Self::from_handle(handle, max_frame)
    }
    fn from_handle(handle: Handle, max_frame: usize) -> io::Result<Self> {
        let enabled = 1i32;
        if unsafe {
            setsockopt(
                handle.0,
                IPPROTO_TCP,
                TCP_NODELAY,
                (&enabled as *const i32).cast(),
                size_of::<i32>() as i32,
            )
        } == SOCKET_ERROR
        {
            return Err(error());
        }
        Ok(Self {
            handle,
            max_frame,
            poisoned: false,
        })
    }
    /// Set a bounded blocking read timeout; None restores an infinite timeout.
    pub fn set_read_timeout(&self, timeout: Option<std::time::Duration>) -> io::Result<()> {
        self.set_timeout(SO_RCVTIMEO, timeout)
    }
    /// Set a bounded blocking write timeout; None restores an infinite timeout.
    pub fn set_write_timeout(&self, timeout: Option<std::time::Duration>) -> io::Result<()> {
        self.set_timeout(SO_SNDTIMEO, timeout)
    }
    fn set_timeout(&self, option: i32, timeout: Option<std::time::Duration>) -> io::Result<()> {
        let millis = match timeout {
            None => 0u32,
            Some(t) if t.is_zero() => return Err(invalid("timeout must be positive")),
            Some(t) => {
                u32::try_from(t.as_millis().max(1)).map_err(|_| invalid("timeout too large"))?
            }
        };
        if unsafe {
            setsockopt(
                self.handle.0,
                SOL_SOCKET,
                option,
                (&millis as *const u32).cast(),
                size_of::<u32>() as i32,
            )
        } == SOCKET_ERROR
        {
            return Err(error());
        }
        Ok(())
    }
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        local_addr(self.handle.0)
    }
    fn check(&self) -> io::Result<()> {
        if self.poisoned {
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "socket framing terminated",
            ))
        } else {
            Ok(())
        }
    }
    fn terminate(&mut self) {
        self.poisoned = true;
        unsafe {
            shutdown(self.handle.0, SD_BOTH);
        }
    }
    /// Read up to capacity directly into uninitialized memory. Zero means EOF (or zero capacity).
    /// # Safety
    /// ptr must address capacity exclusively writable bytes for the duration of the call.
    pub unsafe fn receive_raw(&mut self, ptr: *mut u8, capacity: usize) -> io::Result<usize> {
        self.check()?;
        if capacity == 0 {
            return Ok(0);
        }
        loop {
            let n = recv(
                self.handle.0,
                ptr,
                capacity.min(i32::MAX as usize) as i32,
                0,
            );
            if n != SOCKET_ERROR {
                return Ok(n as usize);
            }
            let e = error();
            if e.raw_os_error() != Some(WSAEINTR) {
                return Err(e);
            }
        }
    }
    pub fn send_raw(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.check()?;
        if bytes.is_empty() {
            return Ok(0);
        }
        loop {
            let n = unsafe {
                send(
                    self.handle.0,
                    bytes.as_ptr(),
                    bytes.len().min(i32::MAX as usize) as i32,
                    0,
                )
            };
            if n != SOCKET_ERROR {
                return Ok(n as usize);
            }
            let e = error();
            if e.raw_os_error() != Some(WSAEINTR) {
                return Err(e);
            }
        }
    }
    unsafe fn exact(&mut self, mut ptr: *mut u8, mut len: usize) -> io::Result<()> {
        while len != 0 {
            let n = self.receive_raw(ptr, len)?;
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "incomplete frame",
                ));
            }
            ptr = ptr.add(n);
            len -= n;
        }
        Ok(())
    }
    fn receive_frame(&mut self, ptr: *mut u8, capacity: usize) -> io::Result<usize> {
        self.check()?;
        let result = (|| {
            let mut prefix = [0u8; 4];
            unsafe {
                self.exact(prefix.as_mut_ptr(), 4)?;
            }
            let len = u32::from_le_bytes(prefix) as usize;
            if len > self.max_frame || len > capacity {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "frame exceeds capacity",
                ));
            }
            unsafe {
                self.exact(ptr, len)?;
            }
            Ok(len)
        })();
        if result.is_err() {
            self.terminate();
        }
        result
    }
    /// Receive one bounded frame straight into a reusable region buffer.
    /// On error, no partial/stale payload is exposed and the framed socket is terminated.
    pub fn receive_into(&mut self, buffer: &mut RegionBuffer<'_>) -> io::Result<usize> {
        buffer.clear();
        let len = self.receive_frame(buffer.as_mut_ptr(), buffer.capacity())?;
        unsafe {
            buffer.set_len(len);
        }
        Ok(len)
    }
    /// Receive directly into an exclusive IPC reservation; publish separately after validation.
    pub fn receive_shared(&mut self, buffer: &mut SharedWrite) -> io::Result<usize> {
        unsafe {
            buffer.set_len(0);
        }
        let len = self.receive_frame(buffer.as_mut_ptr(), buffer.capacity())?;
        unsafe {
            buffer.set_len(len);
        }
        Ok(len)
    }
    /// Header and payload use a stack WSABUF pair, normally one WSASend, without a frame copy.
    /// Partial sends advance the pair; any I/O failure terminates framing.
    pub fn send_frame(&mut self, payload: &[u8]) -> io::Result<()> {
        self.check()?;
        if payload.len() > self.max_frame {
            return Err(invalid("frame exceeds limit"));
        }
        let prefix = (payload.len() as u32).to_le_bytes();
        let mut buffers = [
            WSABUF {
                len: 4,
                buf: prefix.as_ptr().cast_mut(),
            },
            WSABUF {
                len: payload.len() as u32,
                buf: payload.as_ptr().cast_mut(),
            },
        ];
        let mut first = 0usize;
        while first < 2 {
            if buffers[first].len == 0 {
                first += 1;
                continue;
            }
            let mut sent = 0;
            let result = unsafe {
                WSASend(
                    self.handle.0,
                    buffers[first..].as_ptr(),
                    (2 - first) as u32,
                    &mut sent,
                    0,
                    std::ptr::null_mut(),
                    None,
                )
            };
            if result == SOCKET_ERROR {
                let e = error();
                if e.raw_os_error() == Some(WSAEINTR) {
                    continue;
                }
                self.terminate();
                return Err(e);
            }
            if sent == 0 {
                self.terminate();
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "socket sent zero bytes",
                ));
            }
            while first < 2 && sent >= buffers[first].len {
                sent -= buffers[first].len;
                first += 1;
            }
            if first < 2 && sent != 0 {
                unsafe {
                    buffers[first].buf = buffers[first].buf.add(sent as usize);
                }
                buffers[first].len -= sent;
            }
        }
        Ok(())
    }
    /// Encode directly in caller storage, then send its initialized bytes.
    /// # Safety
    /// Every field must satisfy its generated encoder's schema/UTF-8 contracts.
    pub unsafe fn send_record<T: Serializable>(
        &mut self,
        value: &T,
        buffer: &mut RegionBuffer<'_>,
    ) -> io::Result<usize> {
        self.check()?;
        let capacity = value.serialized_size();
        if capacity > buffer.capacity() || capacity > self.max_frame {
            return Err(invalid("insufficient frame capacity"));
        }
        buffer.clear();
        let used = value.serialize_into_unsafe(buffer.as_mut_ptr());
        buffer.set_len(used);
        self.send_frame(buffer)?;
        Ok(used)
    }
    /// Encode a borrowed/guard record directly in the reusable region buffer.
    /// # Safety
    /// Every field must satisfy its generated encoder's schema/UTF-8 contracts.
    pub unsafe fn send_borrowed_record<'a, T: BorrowSerializable<'a>>(
        &mut self,
        value: &T,
        buffer: &mut RegionBuffer<'_>,
    ) -> io::Result<usize> {
        self.check()?;
        let capacity = value.serialized_size();
        if capacity > buffer.capacity() || capacity > self.max_frame {
            return Err(invalid("insufficient frame capacity"));
        }
        buffer.clear();
        let used = value.serialize_into_unsafe(buffer.as_mut_ptr());
        buffer.set_len(used);
        self.send_frame(buffer)?;
        Ok(used)
    }
}

pub struct NativeListener {
    handle: Handle,
    max_frame: usize,
}
impl NativeListener {
    pub fn bind(address: SocketAddr, max_frame: usize, backlog: i32) -> io::Result<Self> {
        validate_limit(max_frame)?;
        if backlog <= 0 {
            return Err(invalid("invalid listen backlog"));
        }
        let handle = Handle::new(address)?;
        // Exclusive bind avoids silently sharing a port with another listener.
        let enabled = 1i32;
        if unsafe {
            setsockopt(
                handle.0,
                SOL_SOCKET,
                SO_EXCLUSIVEADDRUSE,
                (&enabled as *const i32).cast(),
                size_of::<i32>() as i32,
            )
        } == SOCKET_ERROR
        {
            return Err(error());
        }
        if with_addr(address, |a, n| unsafe { bind(handle.0, a, n) }) == SOCKET_ERROR {
            return Err(error());
        }
        if unsafe { listen(handle.0, backlog) } == SOCKET_ERROR {
            return Err(error());
        }
        Ok(Self { handle, max_frame })
    }
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        local_addr(self.handle.0)
    }
    pub fn accept(&self) -> io::Result<NativeSocket> {
        loop {
            let s = unsafe { accept(self.handle.0, std::ptr::null_mut(), std::ptr::null_mut()) };
            if s != INVALID_SOCKET {
                return NativeSocket::from_handle(Handle(s), self.max_frame);
            }
            let e = error();
            if e.raw_os_error() != Some(WSAEINTR) {
                return Err(e);
            }
        }
    }
}
