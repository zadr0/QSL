# Public declaration index

[Русский](API.ru.md) · [Architecture](architecture.en.md)

792 explicitly public declarations and trait method signatures from current sources. This is a source index, not reflection of every platform export. rust-macros/diagnostics feature gates still apply; no native/Windows dependency remains; generator string literals are excluded. Paths/lines refer to the local snapshot in the [manifest](../reference/core-refactor-2026-11-08/source-manifest.json). This index follows current main, including qsl-core extraction after published 0.2.0.

## qsl-core/src/analysis.rs

`qsl-core/src/analysis.rs:5`

```rust
pub enum BitRegister {
```

`qsl-core/src/analysis.rs:14`

```rust
pub fn for_bits(bits: usize) -> QslResult<Self> {
```

`qsl-core/src/analysis.rs:25`

```rust
pub fn bits(self) -> usize {
```

`qsl-core/src/analysis.rs:35`

```rust
pub fn reader(self) -> &'static str {
```

`qsl-core/src/analysis.rs:45`

```rust
pub fn writer(self) -> &'static str {
```

`qsl-core/src/analysis.rs:57`

```rust
pub struct BitBlock {
```

`qsl-core/src/analysis.rs:62`

```rust
pub struct QslIoPlan {
```

`qsl-core/src/analysis.rs:111`

```rust
pub fn analyze(schema: &QslSchema) -> QslResult<Self> {
```

## qsl-core/src/bit_io.rs

`qsl-core/src/bit_io.rs:70`

```rust
pub unsafe fn read_inline_string_into_with<const MAX: usize, H: crate::StringHeader>( &mut self, out: *mut crate::InlineString<MAX>, ) {
```

`qsl-core/src/bit_io.rs:81`

```rust
pub unsafe fn read_raw_inline_string_into_with<const MAX: usize, H: crate::StringHeader>( &mut self, out: *mut crate::InlineString<MAX>, ) {
```

`qsl-core/src/bit_io.rs:92`

```rust
pub unsafe fn read_inline_string_with<const MAX: usize, H: crate::StringHeader>( &mut self, ) -> crate::InlineString<MAX> {
```

`qsl-core/src/bit_io.rs:103`

```rust
pub unsafe fn read_raw_inline_string_with<const MAX: usize, H: crate::StringHeader>( &mut self, ) -> crate::InlineString<MAX> {
```

`qsl-core/src/bit_io.rs:112`

```rust
pub fn as_ptr(&self) -> *const u8 {
```

`qsl-core/src/bit_io.rs:118`

```rust
pub unsafe fn read_exact(&mut self, n: usize) -> *const u8 {
```

`qsl-core/src/bit_io.rs:127`

```rust
pub unsafe fn read_u8(&mut self) -> u8 {
```

`qsl-core/src/bit_io.rs:133`

```rust
pub unsafe fn read_u16(&mut self) -> u16 {
```

`qsl-core/src/bit_io.rs:139`

```rust
pub unsafe fn read_u32(&mut self) -> u32 {
```

`qsl-core/src/bit_io.rs:145`

```rust
pub unsafe fn read_u64(&mut self) -> u64 {
```

`qsl-core/src/bit_io.rs:151`

```rust
pub unsafe fn read_i8(&mut self) -> i8 {
```

`qsl-core/src/bit_io.rs:157`

```rust
pub unsafe fn read_i16(&mut self) -> i16 {
```

`qsl-core/src/bit_io.rs:163`

```rust
pub unsafe fn read_i32(&mut self) -> i32 {
```

`qsl-core/src/bit_io.rs:169`

```rust
pub unsafe fn read_i64(&mut self) -> i64 {
```

`qsl-core/src/bit_io.rs:175`

```rust
pub unsafe fn read_inline_string<const MAX: usize>(&mut self) -> crate::InlineString<MAX> {
```

`qsl-core/src/bit_io.rs:184`

```rust
pub unsafe fn read_inline_string_into<const MAX: usize>( &mut self, out: *mut crate::InlineString<MAX>, ) {
```

`qsl-core/src/bit_io.rs:195`

```rust
pub unsafe fn read_raw_inline_string_into<const MAX: usize>( &mut self, out: *mut crate::InlineString<MAX>, ) {
```

`qsl-core/src/bit_io.rs:210`

```rust
pub unsafe fn write_inline_string_with<const MAX: usize, H: crate::StringHeader>( &mut self, value: &crate::InlineString<MAX>, ) {
```

`qsl-core/src/bit_io.rs:219`

```rust
pub fn as_mut_ptr(&self) -> *mut u8 {
```

`qsl-core/src/bit_io.rs:225`

```rust
pub unsafe fn claim_bytes(&mut self, n: usize) -> *mut u8 {
```

`qsl-core/src/bit_io.rs:234`

```rust
pub unsafe fn write_bytes_unchecked(&mut self, bytes: &[u8]) {
```

`qsl-core/src/bit_io.rs:240`

```rust
pub unsafe fn write_ptr_unchecked(&mut self, source: *const u8, n: usize) {
```

`qsl-core/src/bit_io.rs:247`

```rust
pub unsafe fn write_zeros_unchecked(&mut self, n: usize) {
```

`qsl-core/src/bit_io.rs:254`

```rust
pub unsafe fn write_inline_string<const MAX: usize>( &mut self, value: &crate::InlineString<MAX>, ) {
```

`qsl-core/src/bit_io.rs:264`

```rust
pub struct QslByteReader {
```

`qsl-core/src/bit_io.rs:271`

```rust
pub unsafe fn new(ptr: *const u8) -> Self {
```

`qsl-core/src/bit_io.rs:275`

```rust
pub fn align(&mut self) {
```

`qsl-core/src/bit_io.rs:279`

```rust
pub unsafe fn read_bits(&mut self, width: u32) -> u64 {
```

`qsl-core/src/bit_io.rs:285`

```rust
pub struct QslByteWriter {
```

`qsl-core/src/bit_io.rs:292`

```rust
pub unsafe fn from_raw(ptr: *mut u8) -> Self {
```

`qsl-core/src/bit_io.rs:296`

```rust
pub fn flush_unchecked(&mut self) {
```

`qsl-core/src/bit_io.rs:300`

```rust
pub unsafe fn write_bits_unchecked(&mut self, _value: u64, width: u32) {
```

`qsl-core/src/bit_io.rs:307`

```rust
pub struct $reader {
```

`qsl-core/src/bit_io.rs:316`

```rust
pub unsafe fn new(ptr: *const u8) -> Self {
```

`qsl-core/src/bit_io.rs:326`

```rust
pub unsafe fn align(&mut self) {
```

`qsl-core/src/bit_io.rs:333`

```rust
pub unsafe fn read_bits(&mut self, width: u32) -> u64 {
```

`qsl-core/src/bit_io.rs:348`

```rust
pub struct $writer {
```

`qsl-core/src/bit_io.rs:357`

```rust
pub unsafe fn from_raw(ptr: *mut u8) -> Self {
```

`qsl-core/src/bit_io.rs:367`

```rust
pub unsafe fn write_bits_unchecked(&mut self, value: u64, width: u32) {
```

`qsl-core/src/bit_io.rs:382`

```rust
pub unsafe fn flush_unchecked(&mut self) {
```

## qsl-core/src/bitmap.rs

`qsl-core/src/bitmap.rs:4`

```rust
pub struct BitmapPlan {
```

`qsl-core/src/bitmap.rs:9`

```rust
pub fn new(fields: usize) -> QslResult<Self> {
```

`qsl-core/src/bitmap.rs:21`

```rust
pub fn bytes(self) -> usize {
```

`qsl-core/src/bitmap.rs:24`

```rust
pub fn rust_type(self) -> &'static str {
```

`qsl-core/src/bitmap.rs:34`

```rust
pub fn marker_bit(self, marker: usize) -> QslResult<u128> {
```

`qsl-core/src/bitmap.rs:40`

```rust
pub fn validate(self, mask: u128) -> QslResult<()> {
```

## qsl-core/src/config.rs

`qsl-core/src/config.rs:4`

```rust
pub const DEFAULT_MAX_DATA_SIZE: usize = 65535;
```

`qsl-core/src/config.rs:7`

```rust
pub struct GenerationConfig {
```

## qsl-core/src/hybrid.rs

`qsl-core/src/hybrid.rs:4`

```rust
pub struct HybridPlan {
```

`qsl-core/src/hybrid.rs:13`

```rust
pub fn analyze(s: &QslSchema) -> QslResult<Option<Self>> {
```

## qsl-core/src/io.rs

`qsl-core/src/io.rs:7`

```rust
pub use checked::{QslCheckedBitsReader, QslCheckedBitsWriter, QslCheckedReader, QslCheckedWriter};
```

`qsl-core/src/io.rs:9`

```rust
pub type QslResult<T> = Result<T, QslError>;
```

`qsl-core/src/io.rs:12`

```rust
pub enum QslError {
```

`qsl-core/src/io.rs:35`

```rust
pub struct QslSizer {
```

`qsl-core/src/io.rs:42`

```rust
pub fn bits(&mut self, n: usize) -> QslResult<()> {
```

`qsl-core/src/io.rs:52`

```rust
pub fn align(&mut self) -> QslResult<()> {
```

`qsl-core/src/io.rs:60`

```rust
pub fn bytes(&mut self, n: usize) -> QslResult<()> {
```

`qsl-core/src/io.rs:66`

```rust
pub fn finish(mut self) -> QslResult<usize> {
```

`qsl-core/src/io.rs:75`

```rust
pub struct QslRawWriter {
```

`qsl-core/src/io.rs:83`

```rust
pub unsafe fn new(ptr: *mut u8) -> Self {
```

`qsl-core/src/io.rs:90`

```rust
pub fn from(out: &mut [u8]) -> Self {
```

`qsl-core/src/io.rs:100`

```rust
pub unsafe fn write_u8(&mut self, value: u8) {
```

`qsl-core/src/io.rs:108`

```rust
pub unsafe fn write_u16(&mut self, value: u16) {
```

`qsl-core/src/io.rs:116`

```rust
pub unsafe fn write_u32(&mut self, value: u32) {
```

`qsl-core/src/io.rs:124`

```rust
pub unsafe fn write_u64(&mut self, value: u64) {
```

`qsl-core/src/io.rs:132`

```rust
pub unsafe fn write(&mut self, source: *const u8, len: usize) {
```

`qsl-core/src/io.rs:140`

```rust
pub struct QslRawReader {
```

`qsl-core/src/io.rs:148`

```rust
pub unsafe fn new(ptr: *const u8) -> Self {
```

`qsl-core/src/io.rs:154`

```rust
pub fn from(input: &[u8]) -> Self {
```

`qsl-core/src/io.rs:163`

```rust
pub unsafe fn read_u8(&mut self) -> u8 {
```

`qsl-core/src/io.rs:172`

```rust
pub unsafe fn read_u16(&mut self) -> u16 {
```

`qsl-core/src/io.rs:181`

```rust
pub unsafe fn read_u32(&mut self) -> u32 {
```

`qsl-core/src/io.rs:190`

```rust
pub unsafe fn read_u64(&mut self) -> u64 {
```

`qsl-core/src/io.rs:198`

```rust
pub unsafe fn take(&mut self, len: usize) -> *const u8 {
```

`qsl-core/src/io.rs:208`

```rust
pub struct QslFastBitsReader {
```

`qsl-core/src/io.rs:219`

```rust
pub unsafe fn read_inline_string_into_with<const MAX: usize, H: crate::StringHeader>( &mut self, out: *mut crate::InlineString<MAX>, ) {
```

`qsl-core/src/io.rs:230`

```rust
pub unsafe fn read_raw_inline_string_into_with<const MAX: usize, H: crate::StringHeader>( &mut self, out: *mut crate::InlineString<MAX>, ) {
```

`qsl-core/src/io.rs:241`

```rust
pub unsafe fn read_inline_string_with<const MAX: usize, H: crate::StringHeader>( &mut self, ) -> crate::InlineString<MAX> {
```

`qsl-core/src/io.rs:252`

```rust
pub unsafe fn read_raw_inline_string_with<const MAX: usize, H: crate::StringHeader>( &mut self, ) -> crate::InlineString<MAX> {
```

`qsl-core/src/io.rs:263`

```rust
pub fn as_ptr(&self) -> *const u8 {
```

`qsl-core/src/io.rs:271`

```rust
pub unsafe fn read_raw_inline_string_into<const MAX: usize>( &mut self, output: *mut crate::InlineString<MAX>, ) {
```

`qsl-core/src/io.rs:283`

```rust
pub unsafe fn read_inline_string_into<const MAX: usize>( &mut self, output: *mut crate::InlineString<MAX>, ) {
```

`qsl-core/src/io.rs:295`

```rust
pub unsafe fn read_inline_string<const MAX: usize>(&mut self) -> crate::InlineString<MAX> {
```

`qsl-core/src/io.rs:306`

```rust
pub unsafe fn new(ptr: *const u8) -> Self {
```

`qsl-core/src/io.rs:317`

```rust
pub fn from(input: &[u8]) -> Self {
```

`qsl-core/src/io.rs:331`

```rust
pub unsafe fn read_bits(&mut self, width: u32) -> u64 {
```

`qsl-core/src/io.rs:350`

```rust
pub unsafe fn align(&mut self) {
```

`qsl-core/src/io.rs:359`

```rust
pub unsafe fn read_exact(&mut self, len: usize) -> *const u8 {
```

`qsl-core/src/io.rs:371`

```rust
pub unsafe fn read_u8(&mut self) -> u8 {
```

`qsl-core/src/io.rs:382`

```rust
pub unsafe fn read_u16(&mut self) -> u16 {
```

`qsl-core/src/io.rs:393`

```rust
pub unsafe fn read_u32(&mut self) -> u32 {
```

`qsl-core/src/io.rs:404`

```rust
pub unsafe fn read_u64(&mut self) -> u64 {
```

`qsl-core/src/io.rs:414`

```rust
pub unsafe fn read_i8(&mut self) -> i8 {
```

`qsl-core/src/io.rs:421`

```rust
pub unsafe fn read_i16(&mut self) -> i16 {
```

`qsl-core/src/io.rs:428`

```rust
pub unsafe fn read_i32(&mut self) -> i32 {
```

`qsl-core/src/io.rs:435`

```rust
pub unsafe fn read_i64(&mut self) -> i64 {
```

`qsl-core/src/io.rs:447`

```rust
pub struct QslUncheckedWriter {
```

`qsl-core/src/io.rs:457`

```rust
pub unsafe fn write_inline_string_with<const MAX: usize, H: crate::StringHeader>( &mut self, value: &crate::InlineString<MAX>, ) {
```

`qsl-core/src/io.rs:468`

```rust
pub fn as_mut_ptr(&self) -> *mut u8 {
```

`qsl-core/src/io.rs:475`

```rust
pub unsafe fn write_inline_string<const MAX: usize>( &mut self, value: &crate::InlineString<MAX>, ) {
```

`qsl-core/src/io.rs:486`

```rust
pub unsafe fn from_raw(ptr: *mut u8) -> Self {
```

`qsl-core/src/io.rs:497`

```rust
pub fn from(out: &mut [u8]) -> Self {
```

`qsl-core/src/io.rs:509`

```rust
pub unsafe fn write_bytes_unchecked(&mut self, bytes: &[u8]) {
```

`qsl-core/src/io.rs:519`

```rust
pub unsafe fn write_ptr_unchecked(&mut self, source: *const u8, len: usize) {
```

`qsl-core/src/io.rs:529`

```rust
pub unsafe fn write_zeros_unchecked(&mut self, count: usize) {
```

`qsl-core/src/io.rs:539`

```rust
pub unsafe fn write_bits_unchecked(&mut self, value: u64, width: u32) {
```

`qsl-core/src/io.rs:556`

```rust
pub unsafe fn flush_unchecked(&mut self) {
```

`qsl-core/src/io.rs:571`

```rust
pub unsafe fn claim_bytes(&mut self, n: usize) -> *mut u8 {
```

`qsl-core/src/io.rs:582`

```rust
pub unsafe fn write_bytes(&mut self, bytes: &[u8]) {
```

`qsl-core/src/io.rs:590`

```rust
pub unsafe fn write_zeros(&mut self, n: usize) {
```

`qsl-core/src/io.rs:598`

```rust
pub unsafe fn write_bits(&mut self, v: u64, width: u32) {
```

`qsl-core/src/io.rs:607`

```rust
pub unsafe fn flush(&mut self) {
```

`qsl-core/src/io.rs:614`

```rust
pub unsafe fn write_u8(&mut self, value: u8) {
```

`qsl-core/src/io.rs:624`

```rust
pub unsafe fn write_u16(&mut self, value: u16) {
```

`qsl-core/src/io.rs:634`

```rust
pub unsafe fn write_u32(&mut self, value: u32) {
```

`qsl-core/src/io.rs:644`

```rust
pub unsafe fn write_u64(&mut self, value: u64) {
```

`qsl-core/src/io.rs:653`

```rust
pub unsafe fn write_i8(&mut self, v: i8) {
```

`qsl-core/src/io.rs:660`

```rust
pub unsafe fn write_i16(&mut self, v: i16) {
```

`qsl-core/src/io.rs:667`

```rust
pub unsafe fn write_i32(&mut self, v: i32) {
```

`qsl-core/src/io.rs:674`

```rust
pub unsafe fn write_i64(&mut self, v: i64) {
```

`qsl-core/src/io.rs:681`

```rust
pub type QslWriter = QslRawWriter;
```

`qsl-core/src/io.rs:682`

```rust
pub type QslReader = QslRawReader;
```

`qsl-core/src/io.rs:683`

```rust
pub type QslBitsWriter = QslUncheckedWriter;
```

`qsl-core/src/io.rs:684`

```rust
pub type QslBitsReader = QslFastBitsReader;
```

## qsl-core/src/io_checked.rs

`qsl-core/src/io_checked.rs:9`

```rust
pub struct QslCheckedWriter<'a> {
```

`qsl-core/src/io_checked.rs:17`

```rust
pub fn from(out: &'a mut [u8]) -> Self {
```

`qsl-core/src/io_checked.rs:22`

```rust
pub unsafe fn from_raw_parts(ptr: *mut u8, len: usize) -> Self {
```

`qsl-core/src/io_checked.rs:30`

```rust
pub fn from_raw(out: &'a mut crate::serialize::SerOut) -> Self {
```

`qsl-core/src/io_checked.rs:33`

```rust
pub fn position(&self) -> usize {
```

`qsl-core/src/io_checked.rs:36`

```rust
pub fn write(&mut self, bytes: &[u8]) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:52`

```rust
pub fn write_zeros(&mut self, len: usize) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:65`

```rust
pub fn write_u8(&mut self, v: u8) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:68`

```rust
pub fn write_u16(&mut self, v: u16) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:71`

```rust
pub fn write_u32(&mut self, v: u32) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:74`

```rust
pub fn write_u64(&mut self, v: u64) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:77`

```rust
pub fn write_i8(&mut self, v: i8) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:80`

```rust
pub fn write_i16(&mut self, v: i16) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:83`

```rust
pub fn write_i32(&mut self, v: i32) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:86`

```rust
pub fn write_i64(&mut self, v: i64) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:91`

```rust
pub struct QslCheckedReader<'a> {
```

`qsl-core/src/io_checked.rs:98`

```rust
pub fn new(bytes: &'a [u8]) -> Self {
```

`qsl-core/src/io_checked.rs:103`

```rust
pub unsafe fn from_raw_parts(ptr: *const u8, len: usize) -> Self {
```

`qsl-core/src/io_checked.rs:111`

```rust
pub fn position(&self) -> usize {
```

`qsl-core/src/io_checked.rs:114`

```rust
pub fn read_bytes(&mut self, n: usize) -> QslResult<&'a [u8]> {
```

`qsl-core/src/io_checked.rs:123`

```rust
pub fn read_u8(&mut self) -> QslResult<u8> {
```

`qsl-core/src/io_checked.rs:126`

```rust
pub fn read_u16(&mut self) -> QslResult<u16> {
```

`qsl-core/src/io_checked.rs:129`

```rust
pub fn read_u32(&mut self) -> QslResult<u32> {
```

`qsl-core/src/io_checked.rs:132`

```rust
pub fn read_u64(&mut self) -> QslResult<u64> {
```

`qsl-core/src/io_checked.rs:135`

```rust
pub fn read_i8(&mut self) -> QslResult<i8> {
```

`qsl-core/src/io_checked.rs:138`

```rust
pub fn read_i16(&mut self) -> QslResult<i16> {
```

`qsl-core/src/io_checked.rs:141`

```rust
pub fn read_i32(&mut self) -> QslResult<i32> {
```

`qsl-core/src/io_checked.rs:144`

```rust
pub fn read_i64(&mut self) -> QslResult<i64> {
```

`qsl-core/src/io_checked.rs:151`

```rust
pub struct QslCheckedBitsWriter<'a> {
```

`qsl-core/src/io_checked.rs:160`

```rust
pub fn from(out: &'a mut [u8]) -> Self {
```

`qsl-core/src/io_checked.rs:165`

```rust
pub unsafe fn from_raw_parts(ptr: *mut u8, len: usize) -> Self {
```

`qsl-core/src/io_checked.rs:175`

```rust
pub fn from_raw(out: &'a mut crate::serialize::SerOut) -> Self {
```

`qsl-core/src/io_checked.rs:178`

```rust
pub fn position(&self) -> usize {
```

`qsl-core/src/io_checked.rs:191`

```rust
pub fn write_bits(&mut self, mut value: u64, mut width: u32) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:213`

```rust
pub fn flush(&mut self) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:221`

```rust
pub fn write_bytes(&mut self, bytes: &[u8]) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:238`

```rust
pub fn write_zeros(&mut self, n: usize) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:252`

```rust
pub fn write_u8(&mut self, v: u8) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:255`

```rust
pub fn write_u16(&mut self, v: u16) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:258`

```rust
pub fn write_u32(&mut self, v: u32) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:261`

```rust
pub fn write_u64(&mut self, v: u64) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:264`

```rust
pub fn write_i8(&mut self, v: i8) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:267`

```rust
pub fn write_i16(&mut self, v: i16) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:270`

```rust
pub fn write_i32(&mut self, v: i32) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:273`

```rust
pub fn write_i64(&mut self, v: i64) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:278`

```rust
pub struct QslCheckedBitsReader<'a> {
```

`qsl-core/src/io_checked.rs:287`

```rust
pub fn new(input: &'a [u8]) -> Self {
```

`qsl-core/src/io_checked.rs:292`

```rust
pub unsafe fn from_raw_parts(ptr: *const u8, len: usize) -> Self {
```

`qsl-core/src/io_checked.rs:302`

```rust
pub fn position(&self) -> usize {
```

`qsl-core/src/io_checked.rs:305`

```rust
pub fn read_bits(&mut self, width: u32) -> QslResult<u64> {
```

`qsl-core/src/io_checked.rs:330`

```rust
pub fn align(&mut self) -> QslResult<()> {
```

`qsl-core/src/io_checked.rs:338`

```rust
pub fn read_bytes(&mut self, n: usize) -> QslResult<&'a [u8]> {
```

`qsl-core/src/io_checked.rs:348`

```rust
pub fn read_u8(&mut self) -> QslResult<u8> {
```

`qsl-core/src/io_checked.rs:351`

```rust
pub fn read_u16(&mut self) -> QslResult<u16> {
```

`qsl-core/src/io_checked.rs:354`

```rust
pub fn read_u32(&mut self) -> QslResult<u32> {
```

`qsl-core/src/io_checked.rs:357`

```rust
pub fn read_u64(&mut self) -> QslResult<u64> {
```

`qsl-core/src/io_checked.rs:360`

```rust
pub fn read_i8(&mut self) -> QslResult<i8> {
```

`qsl-core/src/io_checked.rs:363`

```rust
pub fn read_i16(&mut self) -> QslResult<i16> {
```

`qsl-core/src/io_checked.rs:366`

```rust
pub fn read_i32(&mut self) -> QslResult<i32> {
```

`qsl-core/src/io_checked.rs:369`

```rust
pub fn read_i64(&mut self) -> QslResult<i64> {
```

## qsl-core/src/length_graph.rs

`qsl-core/src/length_graph.rs:6`

```rust
pub struct Domain {
```

`qsl-core/src/length_graph.rs:13`

```rust
pub struct LengthNode {
```

`qsl-core/src/length_graph.rs:22`

```rust
pub struct OffsetRecipe {
```

`qsl-core/src/length_graph.rs:29`

```rust
pub struct LengthGraph {
```

`qsl-core/src/length_graph.rs:37`

```rust
pub fn analyze(s: &QslSchema) -> QslResult<Option<Self>> {
```

`qsl-core/src/length_graph.rs:211`

```rust
pub fn event_index(&self, len: usize) -> Option<usize> {
```

## qsl-core/src/lib.rs

`qsl-core/src/lib.rs:2`

```rust
pub mod analysis;
```

`qsl-core/src/lib.rs:3`

```rust
pub mod bit_io;
```

`qsl-core/src/lib.rs:4`

```rust
pub mod bitmap;
```

`qsl-core/src/lib.rs:5`

```rust
pub mod config;
```

`qsl-core/src/lib.rs:6`

```rust
pub mod hybrid;
```

`qsl-core/src/lib.rs:7`

```rust
pub mod io;
```

`qsl-core/src/lib.rs:8`

```rust
pub mod length_graph;
```

`qsl-core/src/lib.rs:9`

```rust
pub mod modernization;
```

`qsl-core/src/lib.rs:10`

```rust
pub mod optional;
```

`qsl-core/src/lib.rs:11`

```rust
pub mod payload_quant;
```

`qsl-core/src/lib.rs:13`

```rust
pub mod qps;
```

`qsl-core/src/lib.rs:14`

```rust
pub mod qps_bytes;
```

`qsl-core/src/lib.rs:15`

```rust
pub mod rt;
```

`qsl-core/src/lib.rs:16`

```rust
pub mod schema_hash;
```

`qsl-core/src/lib.rs:17`

```rust
pub mod schema_modernization;
```

`qsl-core/src/lib.rs:18`

```rust
pub mod schema_payload_quant;
```

`qsl-core/src/lib.rs:19`

```rust
pub mod serialize;
```

`qsl-core/src/lib.rs:20`

```rust
pub mod socket;
```

`qsl-core/src/lib.rs:21`

```rust
pub mod static_index;
```

`qsl-core/src/lib.rs:22`

```rust
pub mod static_optional;
```

`qsl-core/src/lib.rs:23`

```rust
pub mod strings;
```

`qsl-core/src/lib.rs:24`

```rust
pub mod types;
```

`qsl-core/src/lib.rs:25`

```rust
pub mod validation;
```

`qsl-core/src/lib.rs:26`

```rust
pub mod wire;
```

`qsl-core/src/lib.rs:27`

```rust
pub use bit_io::*;
```

`qsl-core/src/lib.rs:28`

```rust
pub use config::GenerationConfig;
```

`qsl-core/src/lib.rs:29`

```rust
pub use io::*;
```

`qsl-core/src/lib.rs:30`

```rust
pub use modernization::{InlineArray, StackString};
```

`qsl-core/src/lib.rs:31`

```rust
pub use optional::OptionalOffsets;
```

`qsl-core/src/lib.rs:32`

```rust
pub use payload_quant::{QuantizedFrame, QuantizedLease, QuantizedOut, QuantizedRegions};
```

`qsl-core/src/lib.rs:33`

```rust
pub use schema_hash::{ SchemaHashAlgorithm, SchemaHashKind, SchemaHashes, SchemaIdentity, SCHEMA_HASH_VERSION, };
```

`qsl-core/src/lib.rs:36`

```rust
pub use serialize::*;
```

`qsl-core/src/lib.rs:37`

```rust
pub use socket::RawSocket;
```

`qsl-core/src/lib.rs:38`

```rust
pub use strings::*;
```

`qsl-core/src/lib.rs:39`

```rust
pub use types::*;
```

`qsl-core/src/lib.rs:40`

```rust
pub use wire::*;
```

## qsl-core/src/modernization.rs

`qsl-core/src/modernization.rs:2`

```rust
pub use crate::schema_modernization::*;
```

`qsl-core/src/modernization.rs:13`

```rust
pub unsafe fn $name<const N: usize>(source: *const u8) -> $word {
```

`qsl-core/src/modernization.rs:33`

```rust
pub unsafe fn load_exact<const N: usize>(source: *const u8) -> u128 {
```

`qsl-core/src/modernization.rs:39`

```rust
pub struct InlineArray<T, const N: usize> {
```

`qsl-core/src/modernization.rs:49`

```rust
pub const fn new() -> Self {
```

`qsl-core/src/modernization.rs:56`

```rust
pub fn len(&self) -> usize {
```

`qsl-core/src/modernization.rs:59`

```rust
pub fn is_empty(&self) -> bool {
```

`qsl-core/src/modernization.rs:62`

```rust
pub fn capacity(&self) -> usize {
```

`qsl-core/src/modernization.rs:65`

```rust
pub fn as_slice(&self) -> &[T] {
```

`qsl-core/src/modernization.rs:68`

```rust
pub fn as_mut_slice(&mut self) -> &mut [T] {
```

`qsl-core/src/modernization.rs:73`

```rust
pub fn push(&mut self, value: T) -> QslResult<()> {
```

`qsl-core/src/modernization.rs:81`

```rust
pub fn into_array(mut self) -> QslResult<[T;
```

`qsl-core/src/modernization.rs:89`

```rust
pub fn clear(&mut self) {
```

`qsl-core/src/modernization.rs:98`

```rust
pub unsafe fn copy_native_into(source: *const u8, len: usize, out: *mut Self) where T: Copy, {
```

`qsl-core/src/modernization.rs:109`

```rust
pub fn try_from_slice(values: &[T]) -> QslResult<Self> where T: Copy, {
```

`qsl-core/src/modernization.rs:127`

```rust
pub fn try_from_iter(values: impl IntoIterator<Item = T>) -> QslResult<Self> {
```

`qsl-core/src/modernization.rs:162`

```rust
pub struct StackString<const N: usize>(InlineArray<u8, N>);
```

`qsl-core/src/modernization.rs:164`

```rust
pub fn new() -> Self {
```

`qsl-core/src/modernization.rs:167`

```rust
pub fn try_from_str(text: &str) -> QslResult<Self> {
```

`qsl-core/src/modernization.rs:173`

```rust
pub unsafe fn copy_into(source: *const u8, len: usize, out: *mut Self) {
```

`qsl-core/src/modernization.rs:176`

```rust
pub fn as_str(&self) -> &str {
```

`qsl-core/src/modernization.rs:179`

```rust
pub fn as_bytes(&self) -> &[u8] {
```

`qsl-core/src/modernization.rs:182`

```rust
pub fn len(&self) -> usize {
```

`qsl-core/src/modernization.rs:185`

```rust
pub fn is_empty(&self) -> bool {
```

`qsl-core/src/modernization.rs:214`

```rust
pub const fn new(value: u32) -> Self {
```

`qsl-core/src/modernization.rs:221`

```rust
pub fn try_take(&self) -> bool {
```

`qsl-core/src/modernization.rs:230`

```rust
pub fn wait(&self, expected: u32) {
```

`qsl-core/src/modernization.rs:240`

```rust
pub fn wake_all(&self) {
```

`qsl-core/src/modernization.rs:300`

```rust
pub unsafe fn load_word(source: *const u8, count: usize) -> u128 {
```

`qsl-core/src/modernization.rs:306`

```rust
pub unsafe fn store_word(destination: *mut u8, value: u128, count: usize) {
```

`qsl-core/src/modernization.rs:311`

```rust
pub struct CachedFrame<const N: usize> {
```

`qsl-core/src/modernization.rs:321`

```rust
pub const fn new() -> Self {
```

`qsl-core/src/modernization.rs:327`

```rust
pub fn as_slice(&self) -> &[u8] {
```

`qsl-core/src/modernization.rs:330`

```rust
pub fn as_mut_ptr(&mut self) -> *mut u8 {
```

`qsl-core/src/modernization.rs:335`

```rust
pub unsafe fn set_len(&mut self, len: usize) {
```

`qsl-core/src/modernization.rs:342`

```rust
pub struct NativeLeaf {
```

`qsl-core/src/modernization.rs:349`

```rust
pub struct NativeLayout {
```

`qsl-core/src/modernization.rs:354`

```rust
pub const EMPTY: Self = Self {
```

`qsl-core/src/modernization.rs:363`

```rust
pub const fn leaf(bytes: usize, kind: u16, elements: usize) -> Self {
```

`qsl-core/src/modernization.rs:374`

```rust
pub const fn fields(fields: &[(usize, Self)]) -> Self {
```

`qsl-core/src/modernization.rs:391`

```rust
pub const fn compatible(&self, other: &Self) -> bool {
```

`qsl-core/src/modernization.rs:404`

```rust
pub const fn same_fields(&self, other: &Self) -> bool {
```

`qsl-core/src/modernization.rs:427`

```rust
pub unsafe trait NativeField: Copy {
```

`qsl-core/src/modernization.rs:429`

```rust
fn validate_native(bytes: &[u8]) -> QslResult<()> {
```

`qsl-core/src/modernization.rs:437`

```rust
unsafe fn validate_native_fields(address: *const u8) -> QslResult<()>;
```

`qsl-core/src/modernization.rs:476`

```rust
pub static ZERO_WORD: [u8;
```

## qsl-core/src/optional.rs

`qsl-core/src/optional.rs:8`

```rust
pub struct OptionalOffsets<'a> {
```

`qsl-core/src/optional.rs:14`

```rust
pub fn parse(table_and_payload: &'a [u8]) -> QslResult<Self> {
```

`qsl-core/src/optional.rs:45`

```rust
pub fn len(&self) -> usize {
```

`qsl-core/src/optional.rs:49`

```rust
pub fn is_empty(&self) -> bool {
```

`qsl-core/src/optional.rs:54`

```rust
pub fn offset(&self, index: usize) -> Option<usize> {
```

`qsl-core/src/optional.rs:68`

```rust
pub fn field_tail(&self, index: usize) -> Option<&'a [u8]> {
```

## qsl-core/src/payload_quant.rs

`qsl-core/src/payload_quant.rs:3`

```rust
pub use crate::schema_payload_quant::{PayloadQuantPlan, QuantField};
```

`qsl-core/src/payload_quant.rs:12`

```rust
pub fn selected_address<T>(mut value: *const T) -> *const T {
```

`qsl-core/src/payload_quant.rs:27`

```rust
pub fn scalar_operand(mut value: u64) -> u64 {
```

`qsl-core/src/payload_quant.rs:42`

```rust
pub unsafe trait QuantizedRecord: Serializable {
```

`qsl-core/src/payload_quant.rs:47`

```rust
fn quant_validate_values(&self) -> QslResult<()>;
```

`qsl-core/src/payload_quant.rs:51`

```rust
unsafe fn quant_write_prepared(&self, out: *mut u8) -> usize;
```

`qsl-core/src/payload_quant.rs:55`

```rust
unsafe fn quant_read_padded(bytes: *const u8, len: usize, out: *mut Self);
```

`qsl-core/src/payload_quant.rs:57`

```rust
pub trait CheckedQuantizedRecord: QuantizedRecord {
```

`qsl-core/src/payload_quant.rs:58`

```rust
fn quant_validate(input: &[u8]) -> QslResult<()>;
```

`qsl-core/src/payload_quant.rs:61`

```rust
pub struct QuantizedFrame<'a, T: QuantizedRecord> {
```

`qsl-core/src/payload_quant.rs:67`

```rust
pub fn try_new(storage: &'a [u8], len: usize) -> QslResult<Self> {
```

`qsl-core/src/payload_quant.rs:84`

```rust
pub unsafe fn from_validated(storage: &[u8], len: usize) -> QuantizedFrame<'_, T> {
```

`qsl-core/src/payload_quant.rs:91`

```rust
pub fn as_slice(&self) -> &[u8] {
```

`qsl-core/src/payload_quant.rs:94`

```rust
pub fn decode_into<'out>(&self, out: &'out mut std::mem::MaybeUninit<T>) -> &'out mut T {
```

`qsl-core/src/payload_quant.rs:100`

```rust
pub fn decode_boxed(&self) -> Box<T> {
```

`qsl-core/src/payload_quant.rs:109`

```rust
pub struct QuantizedOut<T: QuantizedRecord> {
```

`qsl-core/src/payload_quant.rs:115`

```rust
pub fn encode(value: &T) -> QslResult<Self> {
```

`qsl-core/src/payload_quant.rs:131`

```rust
pub fn as_slice(&self) -> &[u8] {
```

`qsl-core/src/payload_quant.rs:134`

```rust
pub fn frame(&self) -> QuantizedFrame<'_, T> {
```

`qsl-core/src/payload_quant.rs:154`

```rust
pub struct QuantizedRegions<T: QuantizedRecord, const CAP: usize, const N: usize> {
```

`qsl-core/src/payload_quant.rs:164`

```rust
pub const fn new() -> Self {
```

`qsl-core/src/payload_quant.rs:170`

```rust
pub fn encode(&'static self, value: &T) -> QslResult<QuantizedLease<T, CAP, N>> {
```

`qsl-core/src/payload_quant.rs:192`

```rust
pub struct QuantizedLease<T: QuantizedRecord, const CAP: usize, const N: usize> {
```

`qsl-core/src/payload_quant.rs:198`

```rust
pub fn as_slice(&self) -> &[u8] {
```

`qsl-core/src/payload_quant.rs:201`

```rust
pub fn frame(&self) -> QuantizedFrame<'_, T> {
```

`qsl-core/src/payload_quant.rs:225`

```rust
pub trait QuantizedDestination<T: QuantizedRecord> {
```

`qsl-core/src/payload_quant.rs:229`

```rust
unsafe fn decode_padded(self, bytes: *const u8, len: usize);
```

## qsl-core/src/primitive_macro.rs

`qsl-core/src/primitive_macro.rs:26`

```rust
macro_rules! qsl_primitive { ... }
```

`qsl-core/src/primitive_macro.rs:92`

```rust
macro_rules! qsl_native_primitive { ... }
```

## qsl-core/src/qps.rs

`qsl-core/src/qps.rs:9`

```rust
pub use crate::payload_quant::{ PayloadQuantPlan as QpsPlan, QuantField as QpsField, QuantizedFrame, QuantizedLease, QuantizedOut, QuantizedRecord, QuantizedRegions, };
```

`qsl-core/src/qps.rs:15`

```rust
pub const QPS_CODEC_REVISION: u32 = 2;
```

`qsl-core/src/qps.rs:17`

```rust
pub use crate::qps_bytes::{ ByteFrame, ByteMode, ByteQps, CheckedBorrowedRecord, CheckedRecord, BYTE_FRAME_HEADER, BYTE_QPS_CODEC_REVISION, };
```

## qsl-core/src/qps_bytes.rs

`qsl-core/src/qps_bytes.rs:7`

```rust
pub const BYTE_FRAME_HEADER: usize = 16;
```

`qsl-core/src/qps_bytes.rs:8`

```rust
pub const BYTE_QPS_CODEC_REVISION: u32 = 3;
```

`qsl-core/src/qps_bytes.rs:245`

```rust
pub enum ByteMode {
```

`qsl-core/src/qps_bytes.rs:251`

```rust
pub struct ByteQps<const Q: usize>;
```

`qsl-core/src/qps_bytes.rs:252`

```rust
pub struct ByteFrame<'a, const Q: usize> {
```

`qsl-core/src/qps_bytes.rs:258`

```rust
pub fn as_slice(&self) -> &[u8] {
```

`qsl-core/src/qps_bytes.rs:261`

```rust
pub fn original_len(&self) -> usize {
```

`qsl-core/src/qps_bytes.rs:264`

```rust
pub fn mode(&self) -> ByteMode {
```

`qsl-core/src/qps_bytes.rs:267`

```rust
pub fn decode_to<'out>(&self, out: &'out mut [u8]) -> QslResult<&'out mut [u8]> {
```

`qsl-core/src/qps_bytes.rs:270`

```rust
pub fn decode_record_into<'out, T: CheckedRecord>( &self, restoration: &mut [u8], out: &'out mut MaybeUninit<T>, ) -> QslResult<&'out mut T> {
```

`qsl-core/src/qps_bytes.rs:283`

```rust
pub unsafe trait CheckedRecord: Serializable {
```

`qsl-core/src/qps_bytes.rs:285`

```rust
fn qps_validate_values(&self) -> QslResult<()>;
```

`qsl-core/src/qps_bytes.rs:286`

```rust
fn qps_validate_wire(input: &[u8]) -> QslResult<()>;
```

`qsl-core/src/qps_bytes.rs:291`

```rust
pub unsafe trait CheckedBorrowedRecord<'a>: BorrowSerializable<'a> {
```

`qsl-core/src/qps_bytes.rs:293`

```rust
fn qps_validate_values(&self) -> QslResult<()>;
```

`qsl-core/src/qps_bytes.rs:294`

```rust
fn qps_validate_wire(input: &[u8]) -> QslResult<()>;
```

`qsl-core/src/qps_bytes.rs:297`

```rust
pub fn max_encoded_len(len: usize) -> QslResult<usize> {
```

`qsl-core/src/qps_bytes.rs:302`

```rust
pub fn inspect(input: &[u8]) -> QslResult<(ByteMode, usize)> {
```

`qsl-core/src/qps_bytes.rs:331`

```rust
pub fn encode_to<'out>(input: &[u8], out: &'out mut [u8]) -> QslResult<ByteFrame<'out, Q>> {
```

`qsl-core/src/qps_bytes.rs:402`

```rust
pub fn decode_to<'out>(input: &[u8], out: &'out mut [u8]) -> QslResult<&'out mut [u8]> {
```

`qsl-core/src/qps_bytes.rs:407`

```rust
pub fn decode_to_with_limit<'out>( input: &[u8], out: &'out mut [u8], maximum: usize, ) -> QslResult<&'out mut [u8]> {
```

`qsl-core/src/qps_bytes.rs:471`

```rust
pub fn encode_borrowed_to<'out, 'a, T: CheckedBorrowedRecord<'a>>( value: &T, scratch: &mut [u8], out: &'out mut [u8], ) -> QslResult<ByteFrame<'out, Q>> {
```

`qsl-core/src/qps_bytes.rs:485`

```rust
pub fn decode_borrowed<'a, T: CheckedBorrowedRecord<'a>>( input: &[u8], restoration: &'a mut [u8], ) -> QslResult<T> {
```

`qsl-core/src/qps_bytes.rs:493`

```rust
pub fn encode_record_to<'out, T: CheckedRecord>( value: &T, scratch: &mut [u8], out: &'out mut [u8], ) -> QslResult<ByteFrame<'out, Q>> {
```

`qsl-core/src/qps_bytes.rs:507`

```rust
pub fn decode_record_into<'out, T: CheckedRecord>( input: &[u8], restoration: &mut [u8], out: &'out mut MaybeUninit<T>, ) -> QslResult<&'out mut T> {
```

## qsl-core/src/rt.rs

`qsl-core/src/rt.rs:12`

```rust
pub struct StaticRegions<const CAP: usize, const N: usize> {
```

`qsl-core/src/rt.rs:26`

```rust
pub const fn new() -> Self {
```

`qsl-core/src/rt.rs:34`

```rust
pub fn try_acquire(&'static self) -> QslResult<RegionLease<CAP, N>> {
```

`qsl-core/src/rt.rs:65`

```rust
pub fn acquire(&'static self) -> RegionLease<CAP, N> {
```

`qsl-core/src/rt.rs:74`

```rust
pub const fn new_zeroed() -> Self {
```

`qsl-core/src/rt.rs:83`

```rust
pub struct RegionLease<const CAP: usize, const N: usize> {
```

`qsl-core/src/rt.rs:89`

```rust
pub fn capacity(&self) -> usize {
```

`qsl-core/src/rt.rs:92`

```rust
pub fn as_slice(&self) -> &[u8] {
```

`qsl-core/src/rt.rs:101`

```rust
pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
```

`qsl-core/src/rt.rs:106`

```rust
pub unsafe fn set_len(&mut self, len: usize) {
```

## qsl-core/src/schema_hash.rs

`qsl-core/src/schema_hash.rs:7`

```rust
pub const SCHEMA_HASH_VERSION: u16 = 1;
```

`qsl-core/src/schema_hash.rs:11`

```rust
pub enum SchemaHashAlgorithm {
```

`qsl-core/src/schema_hash.rs:19`

```rust
pub const fn bits(self) -> u32 {
```

`qsl-core/src/schema_hash.rs:25`

```rust
pub const fn name(self) -> &'static str {
```

`qsl-core/src/schema_hash.rs:31`

```rust
pub const fn rust_variant(self) -> &'static str {
```

`qsl-core/src/schema_hash.rs:56`

```rust
pub enum SchemaHashKind {
```

`qsl-core/src/schema_hash.rs:65`

```rust
pub struct SchemaHashes {
```

`qsl-core/src/schema_hash.rs:75`

```rust
pub fn strict_hex(self) -> String {
```

`qsl-core/src/schema_hash.rs:78`

```rust
pub fn light_hex(self) -> String {
```

`qsl-core/src/schema_hash.rs:90`

```rust
pub trait SchemaIdentity {
```

`qsl-core/src/schema_hash.rs:97`

```rust
pub fn hashes(&self, algorithm: SchemaHashAlgorithm) -> QslResult<SchemaHashes> {
```

`qsl-core/src/schema_hash.rs:112`

```rust
pub fn hash_descriptor(&self, kind: SchemaHashKind) -> QslResult<Vec<u8>> {
```

## qsl-core/src/schema_modernization.rs

`qsl-core/src/schema_modernization.rs:5`

```rust
pub struct SchemaOptions {
```

`qsl-core/src/schema_modernization.rs:21`

```rust
pub fn with_native_overlay(mut self) -> Self {
```

`qsl-core/src/schema_modernization.rs:26`

```rust
pub fn native_overlay(&self) -> bool {
```

`qsl-core/src/schema_modernization.rs:34`

```rust
pub fn register_word(bytes: usize) -> QslResult<&'static str> {
```

`qsl-core/src/schema_modernization.rs:46`

```rust
pub struct RegisterGroup {
```

`qsl-core/src/schema_modernization.rs:54`

```rust
pub fn register_groups(s: &QslSchema) -> QslResult<Vec<RegisterGroup>> {
```

`qsl-core/src/schema_modernization.rs:133`

```rust
pub enum StorageKind {
```

`qsl-core/src/schema_modernization.rs:143`

```rust
pub fn field_rust_type(f: &crate::QslField, borrowed: bool) -> QslResult<String> {
```

`qsl-core/src/schema_modernization.rs:170`

```rust
pub fn lower_storage(s: &QslSchema, limit: usize) -> QslResult<QslSchema> {
```

`qsl-core/src/schema_modernization.rs:238`

```rust
pub fn validate_options(s: &QslSchema) -> Result<(), String> {
```

`qsl-core/src/schema_modernization.rs:369`

```rust
pub struct NativeRegion {
```

`qsl-core/src/schema_modernization.rs:400`

```rust
pub fn check_native_offsets(s: &QslSchema) -> Result<(), String> {
```

`qsl-core/src/schema_modernization.rs:436`

```rust
pub struct OptionalRepresentation {
```

`qsl-core/src/schema_modernization.rs:443`

```rust
pub fn optional_representation( f: &crate::QslField, aggressive: bool, ) -> Option<OptionalRepresentation> {
```

`qsl-core/src/schema_modernization.rs:501`

```rust
pub fn check_storage_limit(s: &QslSchema, limit: usize) -> Result<(), String> {
```

`qsl-core/src/schema_modernization.rs:542`

```rust
pub struct NativeTypeLayout {
```

`qsl-core/src/schema_modernization.rs:550`

```rust
pub fn native_type_layout( t: &QslType, types: &std::collections::BTreeMap<String, NativeTypeLayout>, ) -> Option<NativeTypeLayout> {
```

## qsl-core/src/schema_payload_quant.rs

`qsl-core/src/schema_payload_quant.rs:5`

```rust
pub struct QuantField {
```

`qsl-core/src/schema_payload_quant.rs:17`

```rust
pub struct PayloadQuantPlan {
```

`qsl-core/src/schema_payload_quant.rs:29`

```rust
pub fn analyze(s: &QslSchema) -> QslResult<Self> {
```

## qsl-core/src/serialize.rs

`qsl-core/src/serialize.rs:3`

```rust
pub trait Serializable: Sized {
```

`qsl-core/src/serialize.rs:10`

```rust
fn get_size() -> usize;
```

`qsl-core/src/serialize.rs:15`

```rust
fn serialized_size(&self) -> usize;
```

`qsl-core/src/serialize.rs:22`

```rust
unsafe fn serialize(&self) -> Vec<u8> {
```

`qsl-core/src/serialize.rs:35`

```rust
unsafe fn serialize_unsafe(&self, to: *mut u8);
```

`qsl-core/src/serialize.rs:41`

```rust
unsafe fn serialize_into_unsafe(&self, to: *mut u8) -> usize {
```

`qsl-core/src/serialize.rs:49`

```rust
unsafe fn serialize_to(&self, to: &mut [u8]) -> usize {
```

`qsl-core/src/serialize.rs:56`

```rust
unsafe fn deserialize(input: &[u8]) -> Self {
```

`qsl-core/src/serialize.rs:65`

```rust
unsafe fn deserialize_into(bytes: *const u8, len: usize, out: *mut Self) {
```

`qsl-core/src/serialize.rs:75`

```rust
unsafe fn deserialize_reuse(input: &[u8], output: &mut Self) {
```

`qsl-core/src/serialize.rs:83`

```rust
unsafe fn deserialize_prepared_into(bytes: *const u8, len: usize, output: &mut Self) {
```

`qsl-core/src/serialize.rs:91`

```rust
unsafe fn deserialize_unsafe(bytes: *const u8) -> Self;
```

`qsl-core/src/serialize.rs:96`

```rust
unsafe fn deserialize_from_raw_parts(bytes: *const u8, _len: usize) -> Self {
```

`qsl-core/src/serialize.rs:103`

```rust
pub trait BorrowSerializable<'a>: Sized {
```

`qsl-core/src/serialize.rs:109`

```rust
fn get_size() -> usize;
```

`qsl-core/src/serialize.rs:111`

```rust
fn serialized_size(&self) -> usize;
```

`qsl-core/src/serialize.rs:117`

```rust
unsafe fn serialize(&self) -> Vec<u8> {
```

`qsl-core/src/serialize.rs:129`

```rust
unsafe fn serialize_unsafe(&self, to: *mut u8);
```

`qsl-core/src/serialize.rs:135`

```rust
unsafe fn serialize_into_unsafe(&self, to: *mut u8) -> usize {
```

`qsl-core/src/serialize.rs:142`

```rust
unsafe fn serialize_to(&self, to: &mut [u8]) -> usize {
```

`qsl-core/src/serialize.rs:148`

```rust
unsafe fn deserialize_borrowed(input: &'a [u8]) -> Self {
```

`qsl-core/src/serialize.rs:157`

```rust
unsafe fn deserialize_borrowed_into(bytes: *const u8, len: usize, out: *mut Self) {
```

`qsl-core/src/serialize.rs:164`

```rust
unsafe fn deserialize_borrowed_unsafe(bytes: *const u8) -> Self;
```

`qsl-core/src/serialize.rs:169`

```rust
unsafe fn deserialize_borrowed_from_raw_parts(bytes: *const u8, _len: usize) -> Self {
```

`qsl-core/src/serialize.rs:174`

```rust
pub struct SerOut {
```

`qsl-core/src/serialize.rs:182`

```rust
pub fn new(len: usize) -> Self {
```

`qsl-core/src/serialize.rs:196`

```rust
pub fn with_capacity_aligned(capacity: usize, alignment: usize) -> Self {
```

`qsl-core/src/serialize.rs:213`

```rust
pub fn capacity(&self) -> usize {
```

`qsl-core/src/serialize.rs:217`

```rust
pub fn alignment(&self) -> usize {
```

`qsl-core/src/serialize.rs:221`

```rust
pub fn clear(&mut self) {
```

`qsl-core/src/serialize.rs:228`

```rust
pub unsafe fn set_len(&mut self, len: usize) {
```

`qsl-core/src/serialize.rs:236`

```rust
pub unsafe fn from_raw(ptr: *mut u8, layout: std::alloc::Layout, len: usize) -> Self {
```

`qsl-core/src/serialize.rs:246`

```rust
pub fn as_mut_ptr(&mut self) -> *mut u8 {
```

`qsl-core/src/serialize.rs:291`

```rust
pub fn len(&self) -> usize {
```

`qsl-core/src/serialize.rs:295`

```rust
pub fn is_empty(&self) -> bool {
```

`qsl-core/src/serialize.rs:300`

```rust
pub fn copy_from_slice(&mut self, bytes: &[u8]) -> crate::QslResult<()> {
```

`qsl-core/src/serialize.rs:314`

```rust
pub unsafe fn copy_from_raw_parts(&mut self, source: *const u8, len: usize) {
```

`qsl-core/src/serialize.rs:319`

```rust
pub fn spare_capacity_mut(&mut self) -> &mut [std::mem::MaybeUninit<u8>] {
```

`qsl-core/src/serialize.rs:330`

```rust
pub trait Sealed {
```

`qsl-core/src/serialize.rs:337`

```rust
pub trait DeserializeDestination<T: Serializable>: destination_private::Sealed {
```

`qsl-core/src/serialize.rs:342`

```rust
unsafe fn decode_into(self, bytes: *const u8, len: usize);
```

## qsl-core/src/socket.rs

`qsl-core/src/socket.rs:10`

```rust
pub struct RawSocket {
```

`qsl-core/src/socket.rs:15`

```rust
pub fn new(stream: TcpStream, max_frame: usize) -> io::Result<Self> {
```

`qsl-core/src/socket.rs:34`

```rust
pub fn receive<'a>(&mut self, storage: &'a mut [u8]) -> io::Result<&'a [u8]> {
```

`qsl-core/src/socket.rs:74`

```rust
pub unsafe fn send_record<T: Serializable>( &mut self, value: &T, storage: &mut [u8], ) -> io::Result<usize> {
```

`qsl-core/src/socket.rs:85`

```rust
pub unsafe fn send_borrowed_record<'a, T: BorrowSerializable<'a>>( &mut self, value: &T, storage: &mut [u8], ) -> io::Result<usize> {
```

`qsl-core/src/socket.rs:94`

```rust
pub fn stream(&self) -> &TcpStream {
```

## qsl-core/src/static_index.rs

`qsl-core/src/static_index.rs:6`

```rust
pub const ANALYSIS_MAX_STATES: usize = 4096;
```

`qsl-core/src/static_index.rs:7`

```rust
pub const ANALYSIS_MAX_BYTES: usize = 65535;
```

`qsl-core/src/static_index.rs:10`

```rust
pub struct StateCode {
```

`qsl-core/src/static_index.rs:18`

```rust
pub struct OffsetRow {
```

`qsl-core/src/static_index.rs:24`

```rust
pub struct PredictPlan {
```

`qsl-core/src/static_index.rs:34`

```rust
pub fn analyze(schema: &QslSchema) -> QslResult<Self> {
```

`qsl-core/src/static_index.rs:141`

```rust
pub fn packed_index(&self) -> QslResult<Vec<u32>> {
```

`qsl-core/src/static_index.rs:145`

```rust
pub fn pack_lengths(&self, lengths: &[Option<usize>]) -> QslResult<u32> {
```

`qsl-core/src/static_index.rs:161`

```rust
pub fn unpack_lengths(&self, d: u32) -> QslResult<Vec<Option<usize>>> {
```

`qsl-core/src/static_index.rs:193`

```rust
pub fn quantize_index(index: &[u32], quantum: usize) -> QslResult<Vec<u32>> {
```

`qsl-core/src/static_index.rs:221`

```rust
pub fn lookup_index(table: &[u32], quantum: usize, index: usize) -> Option<u32> {
```

`qsl-core/src/static_index.rs:235`

```rust
pub fn structural_counts(domains: &[Vec<usize>], max_len: usize) -> QslResult<Vec<u64>> {
```

## qsl-core/src/static_optional.rs

`qsl-core/src/static_optional.rs:4`

```rust
pub enum StaticPresence {
```

`qsl-core/src/static_optional.rs:11`

```rust
pub struct StaticOptionalPlan {
```

`qsl-core/src/static_optional.rs:16`

```rust
pub fn analyze(schema: &QslSchema) -> QslResult<Self> {
```

`qsl-core/src/static_optional.rs:73`

```rust
pub fn header_bytes(&self) -> usize {
```

## qsl-core/src/strings.rs

`qsl-core/src/strings.rs:11`

```rust
pub use crate::wire::{STRING_COMPRESSION_MIN_BYTES, STRING_ENCODER_REVISION};
```

`qsl-core/src/strings.rs:16`

```rust
pub trait Sealed {
```

`qsl-core/src/strings.rs:24`

```rust
pub trait StringHeader: header_sealed::Sealed {
```

`qsl-core/src/strings.rs:29`

```rust
unsafe fn load(input: *const u8) -> u32;
```

`qsl-core/src/strings.rs:32`

```rust
unsafe fn store(output: *mut u8, value: u32);
```

`qsl-core/src/strings.rs:60`

```rust
pub struct InlineString<const MAX: usize> {
```

`qsl-core/src/strings.rs:67`

```rust
pub const MAX_BYTES: usize = MAX;
```

`qsl-core/src/strings.rs:69`

```rust
pub const HEADER_BYTES: usize = crate::wire::inline_string_header_bytes(MAX);
```

`qsl-core/src/strings.rs:71`

```rust
pub const OFFSET_BYTES: usize = if MAX <= 255 {
```

`qsl-core/src/strings.rs:76`

```rust
pub const fn new() -> Self {
```

`qsl-core/src/strings.rs:88`

```rust
pub unsafe fn from_str_unchecked(value: &str) -> Self {
```

`qsl-core/src/strings.rs:102`

```rust
pub const fn spaces<const LEN: usize>() -> Self {
```

`qsl-core/src/strings.rs:116`

```rust
pub fn len(&self) -> usize {
```

`qsl-core/src/strings.rs:122`

```rust
pub fn is_empty(&self) -> bool {
```

`qsl-core/src/strings.rs:128`

```rust
pub fn as_str(&self) -> &str {
```

`qsl-core/src/strings.rs:140`

```rust
pub fn wire_capacity(&self) -> usize {
```

`qsl-core/src/strings.rs:149`

```rust
pub unsafe fn encode_to(&self, output: *mut u8) -> usize {
```

`qsl-core/src/strings.rs:157`

```rust
pub unsafe fn encode_to_with<H: StringHeader>(&self, output: *mut u8) -> usize {
```

`qsl-core/src/strings.rs:165`

```rust
pub unsafe fn encode_raw_to(&self, output: *mut u8) -> usize {
```

`qsl-core/src/strings.rs:180`

```rust
pub unsafe fn decode_into(input: *const u8, output: *mut Self) -> usize {
```

`qsl-core/src/strings.rs:190`

```rust
pub unsafe fn decode_into_with<H: StringHeader>(input: *const u8, output: *mut Self) -> usize {
```

`qsl-core/src/strings.rs:201`

```rust
pub unsafe fn decode_raw_into_with<H: StringHeader>( input: *const u8, output: *mut Self, ) -> usize {
```

`qsl-core/src/strings.rs:222`

```rust
pub unsafe fn decode_from_with<H: StringHeader>(input: *const u8) -> (Self, usize) {
```

`qsl-core/src/strings.rs:232`

```rust
pub unsafe fn decode_raw_from_with<H: StringHeader>(input: *const u8) -> (Self, usize) {
```

`qsl-core/src/strings.rs:242`

```rust
pub unsafe fn decode_raw_into(input: *const u8, output: *mut Self) -> usize {
```

`qsl-core/src/strings.rs:259`

```rust
pub unsafe fn decode_from(input: *const u8) -> (Self, usize) {
```

`qsl-core/src/strings.rs:411`

```rust
pub unsafe fn encode_bounded<const MAX: usize>( input: *const u8, len: usize, output: *mut u8, ) -> usize {
```

`qsl-core/src/strings.rs:429`

```rust
pub unsafe fn encode_bounded_with<const MAX: usize, H: StringHeader>( input: *const u8, len: usize, output: *mut u8, ) -> usize {
```

`qsl-core/src/strings.rs:751`

```rust
pub unsafe fn decode_bounded<const MAX: usize>( input: *const u8, output: *mut u8, ) -> (usize, usize) {
```

`qsl-core/src/strings.rs:768`

```rust
pub unsafe fn decode_bounded_with<const MAX: usize, H: StringHeader>( input: *const u8, output: *mut u8, ) -> (usize, usize) {
```

`qsl-core/src/strings.rs:856`

```rust
pub const FIXED_UTF8_CODEC_VERSION: u8 = 2;
```

`qsl-core/src/strings.rs:864`

```rust
pub unsafe fn encode_fixed_utf8<const MAX: usize>(input: *const u8, output: *mut u8) -> usize {
```

`qsl-core/src/strings.rs:886`

```rust
pub unsafe fn decode_fixed_utf8<const MAX: usize>(input: *const u8, output: *mut u8) -> usize {
```

## qsl-core/src/types.rs

`qsl-core/src/types.rs:4`

```rust
pub const FLAG_COMPACT: usize = 0b0000_0001;
```

`qsl-core/src/types.rs:5`

```rust
pub const FLAG_SAFETY: usize = 0b0000_0010;
```

`qsl-core/src/types.rs:6`

```rust
pub const FLAG_ALIGNMENT: usize = 0b0000_0100;
```

`qsl-core/src/types.rs:7`

```rust
pub const FLAG_OPTIONAL_BITMAP: usize = 0b0001_0000;
```

`qsl-core/src/types.rs:8`

```rust
pub const FLAG_OPT_TABLE: usize = 0b0010_0000;
```

`qsl-core/src/types.rs:9`

```rust
pub const FLAG_PREDICT_SEQ: usize = 0b0100_0000;
```

`qsl-core/src/types.rs:11`

```rust
pub const FLAG_FIELD_BORROW: usize = 0b1000_0000;
```

`qsl-core/src/types.rs:12`

```rust
pub const FLAG_LIFETIME: usize = 0b0000_1000;
```

`qsl-core/src/types.rs:13`

```rust
pub const FLAG_STATE_BITS: usize = 1 << 8;
```

`qsl-core/src/types.rs:14`

```rust
pub const FLAG_INLINE: usize = 1 << 9;
```

`qsl-core/src/types.rs:15`

```rust
pub const FLAG_INLINE_ALWAYS: usize = 1 << 10;
```

`qsl-core/src/types.rs:16`

```rust
pub const FLAG_INLINE_NEVER: usize = 1 << 11;
```

`qsl-core/src/types.rs:17`

```rust
pub const STATE_BITS_QUANTUM_SHIFT: usize = 12;
```

`qsl-core/src/types.rs:18`

```rust
pub const STATE_BITS_QUANTUM_MASK: usize = 7 << STATE_BITS_QUANTUM_SHIFT;
```

`qsl-core/src/types.rs:20`

```rust
pub const FLAG_MACRO: usize = 1 << 15;
```

`qsl-core/src/types.rs:22`

```rust
pub const FLAG_OPT_TABLE_LEGACY: usize = 1 << 16;
```

`qsl-core/src/types.rs:23`

```rust
pub const FLAG_PAYLOAD_QUANT: usize = 1 << 17;
```

`qsl-core/src/types.rs:25`

```rust
pub const FLAG_QPS: usize = FLAG_PAYLOAD_QUANT;
```

`qsl-core/src/types.rs:26`

```rust
pub const PAYLOAD_QUANT_SHIFT: usize = 18;
```

`qsl-core/src/types.rs:27`

```rust
pub const PAYLOAD_QUANT_MASK: usize = 3 << PAYLOAD_QUANT_SHIFT;
```

`qsl-core/src/types.rs:34`

```rust
pub trait QslCustom: Sized {
```

`qsl-core/src/types.rs:43`

```rust
fn qsl_validate(_bytes: &[u8], _compact: bool) -> QslResult<()> {
```

`qsl-core/src/types.rs:49`

```rust
fn qsl_validate_values(&self) -> QslResult<()> {
```

`qsl-core/src/types.rs:56`

```rust
unsafe fn qsl_write_unchecked(&self, to: *mut u8);
```

`qsl-core/src/types.rs:61`

```rust
unsafe fn qsl_read_unchecked(from: *const u8) -> Self;
```

`qsl-core/src/types.rs:65`

```rust
fn qsl_size(&self) -> usize {
```

`qsl-core/src/types.rs:71`

```rust
fn qsl_comp_size(&self) -> usize {
```

`qsl-core/src/types.rs:78`

```rust
unsafe fn qsl_comp_write_unchecked(&self, to: *mut u8) {
```

`qsl-core/src/types.rs:85`

```rust
unsafe fn qsl_comp_read_unchecked(from: *const u8) -> Self {
```

`qsl-core/src/types.rs:91`

```rust
pub trait Sealed {
```

`qsl-core/src/types.rs:97`

```rust
pub trait PrimitiveInteger: primitive_integer_seal::Sealed {
```

`qsl-core/src/types.rs:103`

```rust
pub struct PrimitiveRepr {
```

`qsl-core/src/types.rs:110`

```rust
pub fn data(self) -> QslType {
```

`qsl-core/src/types.rs:118`

```rust
pub fn validate(self) -> QslResult<()> {
```

`qsl-core/src/types.rs:128`

```rust
pub unsafe trait QslPrimitive: QslCustom {
```

`qsl-core/src/types.rs:130`

```rust
fn qsl_to_bits(&self) -> u64;
```

`qsl-core/src/types.rs:133`

```rust
fn qsl_from_bits(bits: u64) -> Self;
```

`qsl-core/src/types.rs:137`

```rust
pub enum QslType {
```

`qsl-core/src/types.rs:197`

```rust
pub fn rank_bits(min: i128, max: i128) -> QslResult<usize> {
```

`qsl-core/src/types.rs:207`

```rust
pub fn rank_width(&self, fallback: usize) -> QslResult<usize> {
```

`qsl-core/src/types.rs:217`

```rust
pub fn rank_base(&self) -> i128 {
```

`qsl-core/src/types.rs:226`

```rust
pub fn number16(signed: bool, min: usize) -> Self {
```

`qsl-core/src/types.rs:233`

```rust
pub fn number32(signed: bool, min: usize) -> Self {
```

`qsl-core/src/types.rs:240`

```rust
pub fn number64(signed: bool, min: usize) -> Self {
```

`qsl-core/src/types.rs:247`

```rust
pub fn primitive(&self) -> QslResult<Option<&'static str>> {
```

`qsl-core/src/types.rs:275`

```rust
pub fn rust_type(&self, borrowed: bool) -> QslResult<String> {
```

`qsl-core/src/types.rs:307`

```rust
pub fn fixed_wire_size(&self) -> QslResult<Option<usize>> {
```

`qsl-core/src/types.rs:344`

```rust
pub fn default_compact_bits(&self) -> QslResult<usize> {
```

`qsl-core/src/types.rs:362`

```rust
pub fn validate(&self, borrowed: bool) -> QslResult<()> {
```

`qsl-core/src/types.rs:414`

```rust
pub struct QslField {
```

`qsl-core/src/types.rs:429`

```rust
pub struct QslSchema {
```

`qsl-core/src/types.rs:438`

```rust
pub fn compact(&self) -> bool {
```

`qsl-core/src/types.rs:441`

```rust
pub fn bitmap(&self) -> bool {
```

`qsl-core/src/types.rs:444`

```rust
pub fn opt_table(&self) -> bool {
```

`qsl-core/src/types.rs:448`

```rust
pub fn qps(&self) -> bool {
```

`qsl-core/src/types.rs:451`

```rust
pub fn quantum(&self) -> usize {
```

`qsl-core/src/types.rs:454`

```rust
pub fn quant_requested(&self) -> bool {
```

`qsl-core/src/types.rs:457`

```rust
pub fn payload_quantum(&self) -> usize {
```

`qsl-core/src/types.rs:460`

```rust
pub fn legacy_opt_table(&self) -> bool {
```

`qsl-core/src/types.rs:463`

```rust
pub fn predict_seq(&self) -> bool {
```

`qsl-core/src/types.rs:469`

```rust
pub fn state_bits(&self) -> bool {
```

`qsl-core/src/types.rs:472`

```rust
pub fn state_bits_quantum(&self) -> usize {
```

`qsl-core/src/types.rs:475`

```rust
pub fn predict_table(&self) -> bool {
```

`qsl-core/src/types.rs:479`

```rust
pub fn predicted_opt_table(&self) -> bool {
```

`qsl-core/src/types.rs:482`

```rust
pub fn sequential(&self) -> bool {
```

`qsl-core/src/types.rs:486`

```rust
pub fn lifetime(&self) -> bool {
```

`qsl-core/src/types.rs:489`

```rust
pub fn borrow_strings(&self) -> bool {
```

`qsl-core/src/types.rs:492`

```rust
pub fn optional_count(&self) -> usize {
```

`qsl-core/src/types.rs:496`

```rust
pub fn tag_bits(&self) -> usize {
```

`qsl-core/src/types.rs:504`

```rust
pub fn count_bits(&self) -> usize {
```

`qsl-core/src/types.rs:513`

```rust
pub fn bitmap_width(&self) -> usize {
```

`qsl-core/src/types.rs:516`

```rust
pub fn bitmap_bytes(&self) -> usize {
```

`qsl-core/src/types.rs:520`

```rust
pub fn bitmap_words(&self) -> usize {
```

`qsl-core/src/types.rs:523`

```rust
pub fn optional_tag_base(&self) -> usize {
```

`qsl-core/src/types.rs:529`

```rust
pub fn validate(&self) -> QslResult<()> {
```

`qsl-core/src/types.rs:713`

```rust
pub fn apply_primitive_bindings( schema: &mut QslSchema, bindings: &std::collections::BTreeMap<String, PrimitiveRepr>, ) -> QslResult<()> {
```

## qsl-core/src/validation.rs

`qsl-core/src/validation.rs:6`

```rust
pub use crate::config::DEFAULT_MAX_DATA_SIZE;
```

`qsl-core/src/validation.rs:8`

```rust
pub fn check_size(size: usize, maximum: usize) -> QslResult<()> {
```

`qsl-core/src/validation.rs:17`

```rust
pub struct ValueBudget {
```

`qsl-core/src/validation.rs:21`

```rust
pub fn new(maximum: usize) -> Self {
```

`qsl-core/src/validation.rs:24`

```rust
pub fn charge(&mut self, bytes: usize) -> QslResult<()> {
```

`qsl-core/src/validation.rs:94`

```rust
pub struct ValidationCursor<'a> {
```

`qsl-core/src/validation.rs:100`

```rust
pub fn new(input: &'a [u8]) -> Self {
```

`qsl-core/src/validation.rs:108`

```rust
pub fn with_limit(input: &'a [u8], maximum: usize) -> QslResult<Self> {
```

`qsl-core/src/validation.rs:111`

```rust
pub fn with_budget( input: &'a [u8], maximum_frame: usize, decoded_budget: usize, ) -> QslResult<Self> {
```

`qsl-core/src/validation.rs:123`

```rust
pub fn charge(&mut self, bytes: usize) -> QslResult<()> {
```

`qsl-core/src/validation.rs:126`

```rust
pub fn check_values(&self, bytes: usize, bits: usize) -> QslResult<()> {
```

`qsl-core/src/validation.rs:139`

```rust
pub fn check_allocation(&mut self, bytes: usize) -> QslResult<()> {
```

`qsl-core/src/validation.rs:149`

```rust
pub fn text(&mut self, n: usize, min: usize, max: usize) -> QslResult<&'a str> {
```

`qsl-core/src/validation.rs:156`

```rust
pub fn payload(&mut self, bytes: usize) -> QslResult<&'a [u8]> {
```

`qsl-core/src/validation.rs:160`

```rust
pub fn bits(&mut self, width: usize) -> QslResult<u64> {
```

`qsl-core/src/validation.rs:182`

```rust
pub fn align(&mut self) -> QslResult<()> {
```

`qsl-core/src/validation.rs:189`

```rust
pub fn position(&self) -> usize {
```

`qsl-core/src/validation.rs:192`

```rust
pub fn bytes(&mut self, n: usize) -> QslResult<&'a [u8]> {
```

`qsl-core/src/validation.rs:200`

```rust
pub fn uint(&mut self, n: usize) -> QslResult<u64> {
```

`qsl-core/src/validation.rs:208`

```rust
pub fn utf8(&mut self, n: usize, min: usize, max: usize) -> QslResult<()> {
```

`qsl-core/src/validation.rs:215`

```rust
pub fn finish(mut self) -> QslResult<()> {
```

`qsl-core/src/validation.rs:224`

```rust
pub fn compressed<const MAX: usize>( &mut self, min: usize, raw_only: bool, fixed: bool, ) -> QslResult<()> {
```

`qsl-core/src/validation.rs:235`

```rust
pub fn decode_compressed_owned<const MAX: usize>( &mut self, min: usize, raw_only: bool, fixed: bool, ) -> QslResult<String> {
```

`qsl-core/src/validation.rs:300`

```rust
pub fn decode_compressed<const MAX: usize>( &mut self, min: usize, raw_only: bool, fixed: bool, ) -> QslResult<crate::InlineString<MAX>> {
```

## qsl-core/src/wire.rs

`qsl-core/src/wire.rs:3`

```rust
pub const STRING_COMPRESSION_MIN_BYTES: usize = 32;
```

`qsl-core/src/wire.rs:5`

```rust
pub const STRING_ENCODER_REVISION: u8 = 2;
```

`qsl-core/src/wire.rs:8`

```rust
pub const fn inline_string_header_bytes(max: usize) -> usize {
```

## qsl-interpreter/src/backend.rs

`qsl-interpreter/src/backend.rs:14`

```rust
pub trait InterpreterBuilderExt {
```

`qsl-interpreter/src/backend.rs:15`

```rust
fn use_interpreter(&self) -> QslResult<Vec<u8>>;
```

`qsl-interpreter/src/backend.rs:17`

```rust
fn use_interpenter(&self) -> QslResult<Vec<u8>> {
```

## qsl-interpreter/src/build.rs

`qsl-interpreter/src/build.rs:6`

```rust
pub fn build_qsl_files( qsl_files: &[&str], deq_files: &[&str], out_dir: impl AsRef<Path>, ) -> Result<PathBuf, Box<dyn std::error::Error>> {
```

`qsl-interpreter/src/build.rs:17`

```rust
pub fn build_qsl_files_with_rust( qsl_files: &[&str], deq_files: &[&str], rust_files: &[&str], out_dir: impl AsRef<Path>, ) -> Result<PathBuf, Box<dyn std::error::Error>> {
```

## qsl-interpreter/src/frontend.rs

`qsl-interpreter/src/frontend.rs:5`

```rust
pub struct PreparedSchema {
```

`qsl-interpreter/src/frontend.rs:10`

```rust
pub fn schema(&self) -> &QslSchema {
```

`qsl-interpreter/src/frontend.rs:13`

```rust
pub fn native_overlay(&self) -> bool {
```

`qsl-interpreter/src/frontend.rs:19`

```rust
pub struct PreparedModule {
```

`qsl-interpreter/src/frontend.rs:24`

```rust
pub fn schemas(&self) -> &[PreparedSchema] {
```

`qsl-interpreter/src/frontend.rs:27`

```rust
pub fn config(&self) -> GenerationConfig {
```

`qsl-interpreter/src/frontend.rs:33`

```rust
pub fn prepare_schemas( schemas: &[QslSchema], config: GenerationConfig, ) -> QslResult<PreparedModule> {
```

`qsl-interpreter/src/frontend.rs:40`

```rust
pub fn expand_schemas(schemas: &[QslSchema]) -> QslResult<Vec<QslSchema>> {
```

`qsl-interpreter/src/frontend.rs:44`

```rust
pub fn prepare_expanded( expanded: Vec<QslSchema>, config: GenerationConfig, ) -> QslResult<PreparedModule> {
```

## qsl-interpreter/src/interpreter.rs

`qsl-interpreter/src/interpreter.rs:30`

```rust
pub fn schema_maximum_size(schema: &QslSchema) -> QslResult<Option<usize>> {
```

`qsl-interpreter/src/interpreter.rs:171`

```rust
pub trait TypeInterpreter {
```

`qsl-interpreter/src/interpreter.rs:172`

```rust
fn type_name(&self, out: &mut Vec<u8>, borrowed: bool) -> QslResult<()>;
```

`qsl-interpreter/src/interpreter.rs:173`

```rust
fn size(&self, out: &mut Vec<u8>, expr: &str, width: usize, compact: bool) -> QslResult<()>;
```

`qsl-interpreter/src/interpreter.rs:174`

```rust
fn write(&self, out: &mut Vec<u8>, expr: &str) -> QslResult<()>;
```

`qsl-interpreter/src/interpreter.rs:175`

```rust
fn read(&self, out: &mut Vec<u8>, name: &str, borrowed: bool) -> QslResult<()>;
```

`qsl-interpreter/src/interpreter.rs:176`

```rust
fn comp_write(&self, out: &mut Vec<u8>, expr: &str, bits: usize) -> QslResult<()>;
```

`qsl-interpreter/src/interpreter.rs:177`

```rust
fn comp_read( &self, out: &mut Vec<u8>, name: &str, bits: usize, borrowed: bool, ) -> QslResult<()>;
```

`qsl-interpreter/src/interpreter.rs:2663`

```rust
pub struct QslInterpreter {
```

`qsl-interpreter/src/interpreter.rs:2669`

```rust
pub type RustGenerator = QslInterpreter;
```

`qsl-interpreter/src/interpreter.rs:2694`

```rust
pub fn new() -> Self {
```

`qsl-interpreter/src/interpreter.rs:2699`

```rust
pub fn generate(&mut self, schemas: &[QslSchema]) -> QslResult<()> {
```

`qsl-interpreter/src/interpreter.rs:2707`

```rust
pub fn generate_analyzed(&mut self, module: &crate::AnalyzedModule) -> QslResult<()> {
```

`qsl-interpreter/src/interpreter.rs:2714`

```rust
pub fn generate_prepared(&mut self, module: &crate::frontend::PreparedModule) -> QslResult<()> {
```

`qsl-interpreter/src/interpreter.rs:2758`

```rust
pub fn generate_parsed(&mut self, schemas: &[crate::compat::ParserQslSchema]) -> QslResult<()> {
```

`qsl-interpreter/src/interpreter.rs:2766`

```rust
pub fn generate_field(&mut self, field: &QslField, borrowed: bool) -> QslResult<()> {
```

`qsl-interpreter/src/interpreter.rs:3184`

```rust
pub fn generate_default(&mut self, schema: &QslSchema) -> QslResult<()> {
```

`qsl-interpreter/src/interpreter.rs:3202`

```rust
pub fn generate_lifetime(&mut self, schema: &QslSchema) -> QslResult<()> {
```

`qsl-interpreter/src/interpreter.rs:3208`

```rust
pub fn push(&self) -> std::io::Result<()> {
```

## qsl-interpreter/src/lib.rs

`qsl-interpreter/src/lib.rs:2`

```rust
pub use qsl_lexer::*;
```

`qsl-interpreter/src/lib.rs:5`

```rust
pub use backend::InterpreterBuilderExt;
```

`qsl-interpreter/src/lib.rs:6`

```rust
pub mod frontend;
```

`qsl-interpreter/src/lib.rs:7`

```rust
pub use frontend::{prepare_schemas, PreparedModule, PreparedSchema};
```

`qsl-interpreter/src/lib.rs:8`

```rust
pub mod interpreter;
```

`qsl-interpreter/src/lib.rs:9`

```rust
pub use build::build_qsl_files;
```

`qsl-interpreter/src/lib.rs:11`

```rust
pub use build::build_qsl_files_with_rust;
```

`qsl-interpreter/src/lib.rs:12`

```rust
pub use interpreter::*;
```

`qsl-interpreter/src/lib.rs:20`

```rust
pub mod prelude {
```

`qsl-interpreter/src/lib.rs:21`

```rust
pub use crate::{build_qsl_schem, InterpreterBuilderExt, QslSchemaBuilder, SchemaBackend};
```

## qsl-lexer/src/analysis.rs

`qsl-lexer/src/analysis.rs:2`

```rust
pub use qsl_core::analysis::*;
```

## qsl-lexer/src/bitmap.rs

`qsl-lexer/src/bitmap.rs:2`

```rust
pub use qsl_core::bitmap::*;
```

## qsl-lexer/src/builder.rs

`qsl-lexer/src/builder.rs:2`

```rust
pub use crate::frontend::QslSchemaBuilder;
```

`qsl-lexer/src/builder.rs:13`

```rust
pub trait SchemaBackend {
```

`qsl-lexer/src/builder.rs:16`

```rust
fn compile(&mut self, builder: &QslSchemaBuilder) -> Result<Self::Output, Self::Error>;
```

`qsl-lexer/src/builder.rs:20`

```rust
pub fn new() -> Self {
```

`qsl-lexer/src/builder.rs:28`

```rust
pub fn with_primitive<T: crate::QslPrimitive>(mut self, name: &str) -> QslResult<Self> {
```

`qsl-lexer/src/builder.rs:54`

```rust
pub fn with_source(mut self, source: &str) -> Result<Self, Box<dyn std::error::Error>> {
```

`qsl-lexer/src/builder.rs:69`

```rust
pub fn with_rust_templates( mut self, sources: &[&str], ) -> Result<Self, Box<dyn std::error::Error>> {
```

`qsl-lexer/src/builder.rs:84`

```rust
pub fn from_schemas(schemas: &[QslSchema], config: GenerationConfig) -> QslResult<Self> {
```

`qsl-lexer/src/builder.rs:87`

```rust
pub fn from_source(source: &str) -> Result<Self, Box<dyn std::error::Error>> {
```

`qsl-lexer/src/builder.rs:90`

```rust
pub fn from_source_with_config( source: &str, config: GenerationConfig, ) -> Result<Self, Box<dyn std::error::Error>> {
```

`qsl-lexer/src/builder.rs:98`

```rust
pub fn from_files( qsl_files: &[&str], deq_files: &[&str], ) -> Result<Self, Box<dyn std::error::Error>> {
```

`qsl-lexer/src/builder.rs:105`

```rust
pub fn from_files_with_rust( qsl_files: &[&str], deq_files: &[&str], rust_files: &[&str], ) -> Result<Self, Box<dyn std::error::Error>> {
```

`qsl-lexer/src/builder.rs:114`

```rust
pub fn scan(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
```

`qsl-lexer/src/builder.rs:140`

```rust
pub fn use_backend<B: SchemaBackend>(&self, backend: &mut B) -> Result<B::Output, B::Error> {
```

`qsl-lexer/src/builder.rs:146`

```rust
pub fn build_qsl_schem( path: impl AsRef<Path>, ) -> Result<QslSchemaBuilder, Box<dyn std::error::Error>> {
```

## qsl-lexer/src/compat.rs

`qsl-lexer/src/compat.rs:10`

```rust
pub enum ParserQslType {
```

`qsl-lexer/src/compat.rs:25`

```rust
pub struct ParserQslField {
```

`qsl-lexer/src/compat.rs:34`

```rust
pub struct ParserQslSchema {
```

`qsl-lexer/src/compat.rs:42`

```rust
pub fn new(flags: usize) -> Self {
```

`qsl-lexer/src/compat.rs:48`

```rust
pub fn optional_count(&self) -> usize {
```

`qsl-lexer/src/compat.rs:51`

```rust
pub fn validate(&self) -> QslResult<()> {
```

## qsl-lexer/src/config.rs

`qsl-lexer/src/config.rs:2`

```rust
pub use qsl_core::config::*;
```

## qsl-lexer/src/deq.rs

`qsl-lexer/src/deq.rs:10`

```rust
pub struct DeqConstant {
```

`qsl-lexer/src/deq.rs:16`

```rust
pub struct DeqConfig {
```

`qsl-lexer/src/deq.rs:22`

```rust
pub use crate::config::GenerationConfig;
```

`qsl-lexer/src/deq.rs:27`

```rust
pub fn integer_values(&self) -> HashMap<String, i128> {
```

`qsl-lexer/src/deq.rs:35`

```rust
pub fn parse_deq(source: &str) -> Result<DeqConfig, ParseError> {
```

## qsl-lexer/src/frontend.rs

`qsl-lexer/src/frontend.rs:7`

```rust
pub struct QslSchemaBuilder {
```

`qsl-lexer/src/frontend.rs:13`

```rust
pub fn schemas(&self) -> &[QslSchema] {
```

`qsl-lexer/src/frontend.rs:16`

```rust
pub fn config(&self) -> GenerationConfig {
```

`qsl-lexer/src/frontend.rs:22`

```rust
pub type AnalyzedModule = QslSchemaBuilder;
```

`qsl-lexer/src/frontend.rs:28`

```rust
pub fn analyze_schemas( schemas: &[QslSchema], config: GenerationConfig, ) -> QslResult<AnalyzedModule> {
```

`qsl-lexer/src/frontend.rs:44`

```rust
pub fn expand_schemas(schemas: &[QslSchema]) -> QslResult<Vec<QslSchema>> {
```

## qsl-lexer/src/hybrid.rs

`qsl-lexer/src/hybrid.rs:2`

```rust
pub use qsl_core::hybrid::*;
```

## qsl-lexer/src/io.rs

`qsl-lexer/src/io.rs:2`

```rust
pub use qsl_core::io::{QslError, QslResult, QslSizer};
```

## qsl-lexer/src/length_graph.rs

`qsl-lexer/src/length_graph.rs:2`

```rust
pub use qsl_core::length_graph::*;
```

## qsl-lexer/src/lexer.rs

`qsl-lexer/src/lexer.rs:5`

```rust
pub struct ParseError {
```

`qsl-lexer/src/lexer.rs:19`

```rust
pub fn span(&self) -> (usize, usize) {
```

`qsl-lexer/src/lexer.rs:25`

```rust
pub struct Lexer<'a> {
```

`qsl-lexer/src/lexer.rs:31`

```rust
pub fn new(source: &'a str) -> Self {
```

`qsl-lexer/src/lexer.rs:148`

```rust
pub struct Token<'a> {
```

`qsl-lexer/src/lexer.rs:154`

```rust
pub enum TokenKind {
```

`qsl-lexer/src/lexer.rs:161`

```rust
pub fn byte_offset(&self) -> usize {
```

`qsl-lexer/src/lexer.rs:164`

```rust
pub fn next_token(&mut self) -> Result<Option<Token<'a>>, ParseError> {
```

## qsl-lexer/src/lib.rs

`qsl-lexer/src/lib.rs:2`

```rust
pub mod analysis;
```

`qsl-lexer/src/lib.rs:3`

```rust
pub mod bitmap;
```

`qsl-lexer/src/lib.rs:4`

```rust
pub mod builder;
```

`qsl-lexer/src/lib.rs:5`

```rust
pub mod compat;
```

`qsl-lexer/src/lib.rs:6`

```rust
pub mod config;
```

`qsl-lexer/src/lib.rs:7`

```rust
pub mod deq;
```

`qsl-lexer/src/lib.rs:8`

```rust
pub mod frontend;
```

`qsl-lexer/src/lib.rs:9`

```rust
pub mod hybrid;
```

`qsl-lexer/src/lib.rs:10`

```rust
pub mod io;
```

`qsl-lexer/src/lib.rs:11`

```rust
pub mod length_graph;
```

`qsl-lexer/src/lib.rs:12`

```rust
pub mod lexer;
```

`qsl-lexer/src/lib.rs:13`

```rust
pub mod modernization;
```

`qsl-lexer/src/lib.rs:14`

```rust
pub mod parser;
```

`qsl-lexer/src/lib.rs:15`

```rust
pub mod parser_util;
```

`qsl-lexer/src/lib.rs:16`

```rust
pub mod payload_quant;
```

`qsl-lexer/src/lib.rs:18`

```rust
pub mod regions;
```

`qsl-lexer/src/lib.rs:20`

```rust
pub mod rust_macro;
```

`qsl-lexer/src/lib.rs:21`

```rust
pub mod schema_hash;
```

`qsl-lexer/src/lib.rs:22`

```rust
pub mod schema_macro;
```

`qsl-lexer/src/lib.rs:23`

```rust
pub mod static_index;
```

`qsl-lexer/src/lib.rs:24`

```rust
pub mod static_optional;
```

`qsl-lexer/src/lib.rs:25`

```rust
pub mod types;
```

`qsl-lexer/src/lib.rs:26`

```rust
pub mod wire;
```

`qsl-lexer/src/lib.rs:27`

```rust
pub use analysis::*;
```

`qsl-lexer/src/lib.rs:28`

```rust
pub use builder::{build_qsl_schem, QslSchemaBuilder, SchemaBackend};
```

`qsl-lexer/src/lib.rs:29`

```rust
pub use compat::*;
```

`qsl-lexer/src/lib.rs:30`

```rust
pub use config::GenerationConfig;
```

`qsl-lexer/src/lib.rs:31`

```rust
pub use deq::{parse_deq, DeqConfig, DeqConstant};
```

`qsl-lexer/src/lib.rs:32`

```rust
pub use frontend::{analyze_schemas, AnalyzedModule};
```

`qsl-lexer/src/lib.rs:33`

```rust
pub use io::{QslError, QslResult, QslSizer};
```

`qsl-lexer/src/lib.rs:34`

```rust
pub use lexer::{Lexer, Token, TokenKind};
```

`qsl-lexer/src/lib.rs:35`

```rust
pub use parser::{parse_qsl, parse_qsl_with_constants, ParseError};
```

`qsl-lexer/src/lib.rs:36`

```rust
pub use schema_hash::{ SchemaHashAlgorithm, SchemaHashKind, SchemaHashes, SchemaIdentity, SCHEMA_HASH_VERSION, };
```

`qsl-lexer/src/lib.rs:39`

```rust
pub use types::*;
```

`qsl-lexer/src/lib.rs:40`

```rust
pub use wire::*;
```

`qsl-lexer/src/lib.rs:42`

```rust
pub use qsl_core::{qsl_native_primitive, qsl_primitive};
```

## qsl-lexer/src/modernization.rs

`qsl-lexer/src/modernization.rs:2`

```rust
pub use qsl_core::schema_modernization::*;
```

## qsl-lexer/src/parser.rs

`qsl-lexer/src/parser.rs:5`

```rust
pub use crate::lexer::ParseError;
```

`qsl-lexer/src/parser.rs:107`

```rust
pub fn parse_qsl(source: &str) -> Result<Vec<QslSchema>, ParseError> {
```

`qsl-lexer/src/parser.rs:111`

```rust
pub fn parse_qsl_with_constants( source: &str, globals: &HashMap<String, i128>, ) -> Result<Vec<QslSchema>, ParseError> {
```

`qsl-lexer/src/parser.rs:126`

```rust
pub fn parse_qsl_raw_with_constants( source: &str, globals: &HashMap<String, i128>, ) -> Result<Vec<QslSchema>, ParseError> {
```

`qsl-lexer/src/parser.rs:133`

```rust
pub fn parse_qsl_raw_with_primitives( source: &str, globals: &HashMap<String, i128>, primitives: &std::collections::BTreeMap<String, crate::PrimitiveRepr>, ) -> Result<Vec<QslSchema>, ParseError> {
```

## qsl-lexer/src/parser_util.rs

`qsl-lexer/src/parser_util.rs:4`

```rust
pub struct ParseNum(pub usize, pub bool);
```

`qsl-lexer/src/parser_util.rs:7`

```rust
pub fn rank_bits(min: ParseNum, max: ParseNum) -> Option<usize> {
```

`qsl-lexer/src/parser_util.rs:19`

```rust
pub fn parse_bytes(bytes: &[u8]) -> Option<usize> {
```

`qsl-lexer/src/parser_util.rs:33`

```rust
pub fn parse_hex_bytes(bytes: &[u8]) -> Result<usize, &'static str> {
```

`qsl-lexer/src/parser_util.rs:55`

```rust
pub fn scan_schema_blocks(source: &[u8]) -> Result<Vec<&[u8]>, &'static str> {
```

`qsl-lexer/src/parser_util.rs:74`

```rust
pub fn parse_schema_flags(source: &[u8]) -> Result<usize, &'static str> {
```

`qsl-lexer/src/parser_util.rs:96`

```rust
pub fn parse_primitive_type(token: &[u8]) -> Option<crate::compat::ParserQslType> {
```

`qsl-lexer/src/parser_util.rs:112`

```rust
pub fn encode_signed_minimum(value: ParseNum) -> usize {
```

## qsl-lexer/src/payload_quant.rs

`qsl-lexer/src/payload_quant.rs:2`

```rust
pub use qsl_core::schema_payload_quant::*;
```

## qsl-lexer/src/regions.rs

`qsl-lexer/src/regions.rs:6`

```rust
pub struct RegionError(pub String);
```

`qsl-lexer/src/regions.rs:221`

```rust
pub fn lower_regions(schemas: &[QslSchema]) -> Result<Vec<QslSchema>, RegionError> {
```

## qsl-lexer/src/rust_macro.rs

`qsl-lexer/src/rust_macro.rs:223`

```rust
pub fn parse_rust_macros(sources: &[&str]) -> Result<Vec<QslSchema>, MacroError> {
```

`qsl-lexer/src/rust_macro.rs:357`

```rust
pub fn expand_with_rust( schemas: &[QslSchema], sources: &[&str], ) -> Result<Vec<QslSchema>, MacroError> {
```

`qsl-lexer/src/rust_macro.rs:367`

```rust
pub fn parse_qsl_with_rust( source: &str, constants: &HashMap<String, i128>, rust_sources: &[&str], ) -> Result<Vec<QslSchema>, MacroError> {
```

## qsl-lexer/src/schema_hash.rs

`qsl-lexer/src/schema_hash.rs:2`

```rust
pub use qsl_core::schema_hash::*;
```

## qsl-lexer/src/schema_macro.rs

`qsl-lexer/src/schema_macro.rs:7`

```rust
pub struct MacroError(pub String);
```

`qsl-lexer/src/schema_macro.rs:15`

```rust
pub fn expand_macros(schemas: &[QslSchema]) -> Result<Vec<QslSchema>, MacroError> {
```

## qsl-lexer/src/static_index.rs

`qsl-lexer/src/static_index.rs:2`

```rust
pub use qsl_core::static_index::*;
```

## qsl-lexer/src/static_optional.rs

`qsl-lexer/src/static_optional.rs:2`

```rust
pub use qsl_core::static_optional::*;
```

## qsl-lexer/src/types.rs

`qsl-lexer/src/types.rs:2`

```rust
pub use qsl_core::types::*;
```

## qsl-lexer/src/wire.rs

`qsl-lexer/src/wire.rs:2`

```rust
pub use qsl_core::wire::*;
```

## qsl/macros/src/lib.rs

`qsl/macros/src/lib.rs:91`

```rust
pub fn qsl_macro(args: TokenStream, item: TokenStream) -> TokenStream {
```

`qsl/macros/src/lib.rs:95`

```rust
pub fn schem_macro(args: TokenStream, item: TokenStream) -> TokenStream {
```

## qsl/src/bit_io.rs

`qsl/src/bit_io.rs:2`

```rust
pub use qsl_core::bit_io::*;
```

## qsl/src/deq.rs

`qsl/src/deq.rs:2`

```rust
pub use qsl_interpreter::build_qsl_files;
```

`qsl/src/deq.rs:4`

```rust
pub use qsl_interpreter::build_qsl_files_with_rust;
```

`qsl/src/deq.rs:5`

```rust
pub use qsl_lexer::deq::*;
```

## qsl/src/io.rs

`qsl/src/io.rs:2`

```rust
pub use qsl_core::io::*;
```

## qsl/src/io_checked.rs

`qsl/src/io_checked.rs:2`

```rust
pub use qsl_core::io::*;
```

## qsl/src/lib.rs

`qsl/src/lib.rs:2`

```rust
pub use qsl_interpreter::interpreter;
```

`qsl/src/lib.rs:3`

```rust
pub use qsl_interpreter::InterpreterBuilderExt;
```

`qsl/src/lib.rs:4`

```rust
pub use qsl_lexer::{build_qsl_schem, QslSchemaBuilder, SchemaBackend};
```

`qsl/src/lib.rs:5`

```rust
pub mod prelude {
```

`qsl/src/lib.rs:6`

```rust
pub use crate::{build_qsl_schem, InterpreterBuilderExt, QslSchemaBuilder, SchemaBackend};
```

`qsl/src/lib.rs:8`

```rust
pub mod frontend {
```

`qsl/src/lib.rs:9`

```rust
pub use qsl_interpreter::frontend::{prepare_schemas, PreparedModule, PreparedSchema};
```

`qsl/src/lib.rs:10`

```rust
pub use qsl_lexer::frontend::*;
```

`qsl/src/lib.rs:12`

```rust
pub use qsl_interpreter::{prepare_schemas, PreparedModule, PreparedSchema};
```

`qsl/src/lib.rs:14`

```rust
pub use qsl_lexer::rust_macro;
```

`qsl/src/lib.rs:15`

```rust
pub use qsl_lexer::{ analysis, builder, compat, config, hybrid, length_graph, lexer, parser, parser_util, schema_hash, schema_macro, static_index, static_optional, types, };
```

`qsl/src/lib.rs:19`

```rust
pub mod bit_io;
```

`qsl/src/lib.rs:20`

```rust
pub mod deq;
```

`qsl/src/lib.rs:21`

```rust
pub mod io;
```

`qsl/src/lib.rs:22`

```rust
pub mod modernization;
```

`qsl/src/lib.rs:23`

```rust
pub mod optional;
```

`qsl/src/lib.rs:24`

```rust
pub mod payload_quant;
```

`qsl/src/lib.rs:25`

```rust
pub mod qps;
```

`qsl/src/lib.rs:26`

```rust
pub mod qps_bytes;
```

`qsl/src/lib.rs:27`

```rust
pub mod rt;
```

`qsl/src/lib.rs:28`

```rust
pub mod serialize;
```

`qsl/src/lib.rs:29`

```rust
pub mod socket;
```

`qsl/src/lib.rs:30`

```rust
pub mod strings;
```

`qsl/src/lib.rs:31`

```rust
pub mod validation;
```

`qsl/src/lib.rs:32`

```rust
pub use bit_io::*;
```

`qsl/src/lib.rs:34`

```rust
pub use deq::build_qsl_files_with_rust;
```

`qsl/src/lib.rs:35`

```rust
pub use deq::{build_qsl_files, parse_deq, DeqConfig, DeqConstant};
```

`qsl/src/lib.rs:36`

```rust
pub use interpreter::*;
```

`qsl/src/lib.rs:37`

```rust
pub use io::*;
```

`qsl/src/lib.rs:38`

```rust
pub use modernization::{InlineArray, StackString};
```

`qsl/src/lib.rs:39`

```rust
pub use optional::OptionalOffsets;
```

`qsl/src/lib.rs:40`

```rust
pub use payload_quant::{QuantizedFrame, QuantizedLease, QuantizedOut, QuantizedRegions};
```

`qsl/src/lib.rs:41`

```rust
pub use qsl_lexer::{ analysis::*, compat::*, config::GenerationConfig, frontend::{analyze_schemas, AnalyzedModule}, lexer::{Lexer, Token, TokenKind}, parser::{parse_qsl, parse_qsl_with_constants, ParseError}, schema_hash::{ SchemaHashAlgorithm, SchemaHashKind, SchemaHashes, SchemaIdentity, SCHEMA_HASH_VERSION, }, types::*, };
```

`qsl/src/lib.rs:53`

```rust
pub use qsl_lexer::{qsl_native_primitive, qsl_primitive};
```

`qsl/src/lib.rs:54`

```rust
pub use qsl_macros_impl::{qsl_macro, schem_macro};
```

`qsl/src/lib.rs:55`

```rust
pub use serialize::*;
```

`qsl/src/lib.rs:56`

```rust
pub use socket::RawSocket;
```

`qsl/src/lib.rs:57`

```rust
pub use strings::*;
```

## qsl/src/modernization.rs

`qsl/src/modernization.rs:2`

```rust
pub use qsl_core::modernization::*;
```

## qsl/src/optional.rs

`qsl/src/optional.rs:2`

```rust
pub use qsl_core::optional::*;
```

## qsl/src/payload_quant.rs

`qsl/src/payload_quant.rs:2`

```rust
pub use qsl_core::payload_quant::*;
```

## qsl/src/qps.rs

`qsl/src/qps.rs:2`

```rust
pub use qsl_core::qps::*;
```

## qsl/src/qps_bytes.rs

`qsl/src/qps_bytes.rs:2`

```rust
pub use qsl_core::qps_bytes::*;
```

## qsl/src/rt.rs

`qsl/src/rt.rs:2`

```rust
pub use qsl_core::rt::*;
```

## qsl/src/serialize.rs

`qsl/src/serialize.rs:2`

```rust
pub use qsl_core::serialize::*;
```

## qsl/src/socket.rs

`qsl/src/socket.rs:2`

```rust
pub use qsl_core::socket::*;
```

## qsl/src/strings.rs

`qsl/src/strings.rs:2`

```rust
pub use qsl_core::strings::*;
```

## qsl/src/validation.rs

`qsl/src/validation.rs:2`

```rust
pub use qsl_core::validation::*;
```
