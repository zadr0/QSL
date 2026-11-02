//! Optional, bounds-checked diagnostic I/O. Not part of QSL's hot path.
//! Enable with the Cargo `diagnostics` feature when inspecting untrusted data.

use super::{QslError, QslResult};
use std::marker::PhantomData;
use std::ptr;

/// All writes use raw pointer copies. Safe constructors still bound-check them.
pub struct QslCheckedWriter<'a> {
    ptr: *mut u8,
    len: usize,
    pos: usize,
    _borrow: PhantomData<&'a mut [u8]>,
}

impl<'a> QslCheckedWriter<'a> {
    pub fn from(out: &'a mut [u8]) -> Self {
        unsafe { Self::from_raw_parts(out.as_mut_ptr(), out.len()) }
    }
    /// # Safety
    /// `ptr` must be writable for `len` bytes, exclusively, throughout `'a`.
    pub unsafe fn from_raw_parts(ptr: *mut u8, len: usize) -> Self {
        Self {
            ptr,
            len,
            pos: 0,
            _borrow: PhantomData,
        }
    }
    #[cfg(feature = "native")]
    pub fn from_raw(out: &'a mut crate::serialize::SerOut) -> Self {
        Self::from(out)
    }
    pub fn position(&self) -> usize {
        self.pos
    }
    pub fn write(&mut self, bytes: &[u8]) -> QslResult<()> {
        let end = self
            .pos
            .checked_add(bytes.len())
            .ok_or(QslError::LengthOverflow)?;
        if end > self.len {
            return Err(QslError::BufferTooSmall);
        }
        if !bytes.is_empty() {
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), self.ptr.add(self.pos), bytes.len());
            }
        }
        self.pos = end;
        Ok(())
    }
    pub fn write_zeros(&mut self, len: usize) -> QslResult<()> {
        let end = self.pos.checked_add(len).ok_or(QslError::LengthOverflow)?;
        if end > self.len {
            return Err(QslError::BufferTooSmall);
        }
        if len != 0 {
            unsafe {
                ptr::write_bytes(self.ptr.add(self.pos), 0, len);
            }
        }
        self.pos = end;
        Ok(())
    }
    pub fn write_u8(&mut self, v: u8) -> QslResult<()> {
        self.write(&v.to_le_bytes())
    }
    pub fn write_u16(&mut self, v: u16) -> QslResult<()> {
        self.write(&v.to_le_bytes())
    }
    pub fn write_u32(&mut self, v: u32) -> QslResult<()> {
        self.write(&v.to_le_bytes())
    }
    pub fn write_u64(&mut self, v: u64) -> QslResult<()> {
        self.write(&v.to_le_bytes())
    }
    pub fn write_i8(&mut self, v: i8) -> QslResult<()> {
        self.write(&v.to_le_bytes())
    }
    pub fn write_i16(&mut self, v: i16) -> QslResult<()> {
        self.write(&v.to_le_bytes())
    }
    pub fn write_i32(&mut self, v: i32) -> QslResult<()> {
        self.write(&v.to_le_bytes())
    }
    pub fn write_i64(&mut self, v: i64) -> QslResult<()> {
        self.write(&v.to_le_bytes())
    }
}

pub struct QslCheckedReader<'a> {
    ptr: *const u8,
    len: usize,
    pos: usize,
    _borrow: PhantomData<&'a [u8]>,
}
impl<'a> QslCheckedReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        unsafe { Self::from_raw_parts(bytes.as_ptr(), bytes.len()) }
    }
    /// # Safety
    /// `ptr` must be readable for `len` bytes throughout `'a`.
    pub unsafe fn from_raw_parts(ptr: *const u8, len: usize) -> Self {
        Self {
            ptr,
            len,
            pos: 0,
            _borrow: PhantomData,
        }
    }
    pub fn position(&self) -> usize {
        self.pos
    }
    pub fn read_bytes(&mut self, n: usize) -> QslResult<&'a [u8]> {
        let end = self.pos.checked_add(n).ok_or(QslError::LengthOverflow)?;
        if end > self.len {
            return Err(QslError::UnexpectedEof);
        }
        let result = unsafe { std::slice::from_raw_parts(self.ptr.add(self.pos), n) };
        self.pos = end;
        Ok(result)
    }
    pub fn read_u8(&mut self) -> QslResult<u8> {
        Ok(self.read_bytes(1)?[0])
    }
    pub fn read_u16(&mut self) -> QslResult<u16> {
        Ok(u16::from_le_bytes(self.read_bytes(2)?.try_into().unwrap()))
    }
    pub fn read_u32(&mut self) -> QslResult<u32> {
        Ok(u32::from_le_bytes(self.read_bytes(4)?.try_into().unwrap()))
    }
    pub fn read_u64(&mut self) -> QslResult<u64> {
        Ok(u64::from_le_bytes(self.read_bytes(8)?.try_into().unwrap()))
    }
    pub fn read_i8(&mut self) -> QslResult<i8> {
        Ok(i8::from_le_bytes(self.read_bytes(1)?.try_into().unwrap()))
    }
    pub fn read_i16(&mut self) -> QslResult<i16> {
        Ok(i16::from_le_bytes(self.read_bytes(2)?.try_into().unwrap()))
    }
    pub fn read_i32(&mut self) -> QslResult<i32> {
        Ok(i32::from_le_bytes(self.read_bytes(4)?.try_into().unwrap()))
    }
    pub fn read_i64(&mut self) -> QslResult<i64> {
        Ok(i64::from_le_bytes(self.read_bytes(8)?.try_into().unwrap()))
    }
}

/// LSB-first bit stream. A value always occupies exactly `width` bits,
/// independently of how many leading zeroes it contains.
pub struct QslCheckedBitsWriter<'a> {
    ptr: *mut u8,
    len: usize,
    pos: usize,
    reg: u8,
    used: u32,
    _borrow: PhantomData<&'a mut [u8]>,
}
impl<'a> QslCheckedBitsWriter<'a> {
    pub fn from(out: &'a mut [u8]) -> Self {
        unsafe { Self::from_raw_parts(out.as_mut_ptr(), out.len()) }
    }
    /// # Safety
    /// `ptr` must be exclusively writable for `len` bytes throughout `'a`.
    pub unsafe fn from_raw_parts(ptr: *mut u8, len: usize) -> Self {
        Self {
            ptr,
            len,
            pos: 0,
            reg: 0,
            used: 0,
            _borrow: PhantomData,
        }
    }
    #[cfg(feature = "native")]
    pub fn from_raw(out: &'a mut crate::serialize::SerOut) -> Self {
        Self::from(out)
    }
    pub fn position(&self) -> usize {
        self.pos
    }
    fn push_byte(&mut self, value: u8) -> QslResult<()> {
        if self.pos >= self.len {
            return Err(QslError::BufferTooSmall);
        }
        unsafe {
            *self.ptr.add(self.pos) = value;
        }
        self.pos += 1;
        Ok(())
    }
    pub fn write_bits(&mut self, mut value: u64, mut width: u32) -> QslResult<()> {
        if width > 64 {
            return Err(QslError::InvalidWidth);
        }
        if width < 64 && value >> width != 0 {
            return Err(QslError::InvalidValue);
        }
        while width != 0 {
            let take = (8 - self.used).min(width);
            let mask = (1u16 << take) - 1;
            self.reg |= ((value & u64::from(mask)) as u8) << self.used;
            self.used += take;
            width -= take;
            value >>= take;
            if self.used == 8 {
                self.push_byte(self.reg)?;
                self.reg = 0;
                self.used = 0;
            }
        }
        Ok(())
    }
    pub fn flush(&mut self) -> QslResult<()> {
        if self.used != 0 {
            self.push_byte(self.reg)?;
            self.reg = 0;
            self.used = 0;
        }
        Ok(())
    }
    pub fn write_bytes(&mut self, bytes: &[u8]) -> QslResult<()> {
        self.flush()?;
        let end = self
            .pos
            .checked_add(bytes.len())
            .ok_or(QslError::LengthOverflow)?;
        if end > self.len {
            return Err(QslError::BufferTooSmall);
        }
        if !bytes.is_empty() {
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), self.ptr.add(self.pos), bytes.len());
            }
        }
        self.pos = end;
        Ok(())
    }
    pub fn write_zeros(&mut self, n: usize) -> QslResult<()> {
        self.flush()?;
        let end = self.pos.checked_add(n).ok_or(QslError::LengthOverflow)?;
        if end > self.len {
            return Err(QslError::BufferTooSmall);
        }
        if n != 0 {
            unsafe {
                ptr::write_bytes(self.ptr.add(self.pos), 0, n);
            }
        }
        self.pos = end;
        Ok(())
    }
    pub fn write_u8(&mut self, v: u8) -> QslResult<()> {
        self.write_bytes(&v.to_le_bytes())
    }
    pub fn write_u16(&mut self, v: u16) -> QslResult<()> {
        self.write_bytes(&v.to_le_bytes())
    }
    pub fn write_u32(&mut self, v: u32) -> QslResult<()> {
        self.write_bytes(&v.to_le_bytes())
    }
    pub fn write_u64(&mut self, v: u64) -> QslResult<()> {
        self.write_bytes(&v.to_le_bytes())
    }
    pub fn write_i8(&mut self, v: i8) -> QslResult<()> {
        self.write_bytes(&v.to_le_bytes())
    }
    pub fn write_i16(&mut self, v: i16) -> QslResult<()> {
        self.write_bytes(&v.to_le_bytes())
    }
    pub fn write_i32(&mut self, v: i32) -> QslResult<()> {
        self.write_bytes(&v.to_le_bytes())
    }
    pub fn write_i64(&mut self, v: i64) -> QslResult<()> {
        self.write_bytes(&v.to_le_bytes())
    }
}

pub struct QslCheckedBitsReader<'a> {
    ptr: *const u8,
    len: usize,
    pos: usize,
    reg: u8,
    available: u32,
    _borrow: PhantomData<&'a [u8]>,
}
impl<'a> QslCheckedBitsReader<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        unsafe { Self::from_raw_parts(input.as_ptr(), input.len()) }
    }
    /// # Safety
    /// `ptr` must be valid for `len` readable bytes for `'a`.
    pub unsafe fn from_raw_parts(ptr: *const u8, len: usize) -> Self {
        Self {
            ptr,
            len,
            pos: 0,
            reg: 0,
            available: 0,
            _borrow: PhantomData,
        }
    }
    pub fn position(&self) -> usize {
        self.pos
    }
    pub fn read_bits(&mut self, width: u32) -> QslResult<u64> {
        if width > 64 {
            return Err(QslError::InvalidWidth);
        }
        let mut result = 0u64;
        let mut shifted = 0u32;
        while shifted < width {
            if self.available == 0 {
                if self.pos >= self.len {
                    return Err(QslError::UnexpectedEof);
                }
                self.reg = unsafe { *self.ptr.add(self.pos) };
                self.pos += 1;
                self.available = 8;
            }
            let take = self.available.min(width - shifted);
            let mask = ((1u16 << take) - 1) as u8;
            result |= u64::from(self.reg & mask) << shifted;
            self.reg = if take == 8 { 0 } else { self.reg >> take };
            self.available -= take;
            shifted += take;
        }
        Ok(result)
    }
    /// Ignore only zero-filled alignment bits.
    pub fn align(&mut self) -> QslResult<()> {
        if self.reg != 0 {
            return Err(QslError::InvalidPadding);
        }
        self.reg = 0;
        self.available = 0;
        Ok(())
    }
    pub fn read_bytes(&mut self, n: usize) -> QslResult<&'a [u8]> {
        self.align()?;
        let end = self.pos.checked_add(n).ok_or(QslError::LengthOverflow)?;
        if end > self.len {
            return Err(QslError::UnexpectedEof);
        }
        let value = unsafe { std::slice::from_raw_parts(self.ptr.add(self.pos), n) };
        self.pos = end;
        Ok(value)
    }
    pub fn read_u8(&mut self) -> QslResult<u8> {
        Ok(self.read_bytes(1)?[0])
    }
    pub fn read_u16(&mut self) -> QslResult<u16> {
        Ok(u16::from_le_bytes(self.read_bytes(2)?.try_into().unwrap()))
    }
    pub fn read_u32(&mut self) -> QslResult<u32> {
        Ok(u32::from_le_bytes(self.read_bytes(4)?.try_into().unwrap()))
    }
    pub fn read_u64(&mut self) -> QslResult<u64> {
        Ok(u64::from_le_bytes(self.read_bytes(8)?.try_into().unwrap()))
    }
    pub fn read_i8(&mut self) -> QslResult<i8> {
        Ok(i8::from_le_bytes(self.read_bytes(1)?.try_into().unwrap()))
    }
    pub fn read_i16(&mut self) -> QslResult<i16> {
        Ok(i16::from_le_bytes(self.read_bytes(2)?.try_into().unwrap()))
    }
    pub fn read_i32(&mut self) -> QslResult<i32> {
        Ok(i32::from_le_bytes(self.read_bytes(4)?.try_into().unwrap()))
    }
    pub fn read_i64(&mut self) -> QslResult<i64> {
        Ok(i64::from_le_bytes(self.read_bytes(8)?.try_into().unwrap()))
    }
}
