
Trusted trait-методы Serializable::serialize / BorrowSerializable::serialize теперь unsafe. Их предусловия должны выполняться вызывающим кодом. Сгенерированные RT/QPS serialize() остаются безопасными Result-методами с проверкой значений; для общего безопасного выхода используйте serialize_checked.

> **Контракт 06.11.2026:** [строгий STACK_LIMIT, Safety opt-in, QPS только для чисел, native COPY/LIFETIME](modernization-2026-11-06/MODERNIZATION.ru.md). [Четыре свежих замера](../qsl-bench/modernization/report.html). Таблицы старых измерений сохраняют исходные версии.

# Индекс публичных объявлений

[English](API.en.md) · [Архитектура](architecture.ru.md)

Найдено 625 явно публичных объявлений и сигнатур методов trait в текущих исходниках. Это source-index, а не отражение всех платформенных export-символов. Видимость feature native/Windows и rust-macros сохраняется; кодогенераторские строковые литералы не считаются API. Пути и строки относятся к локальному снимку из [manifest](../reference/optional-modes-source-manifest.json). Исходники и Cargo-манифест крейта находятся в qsl.

## qsl/macros/src/lib.rs

`qsl/macros/src/lib.rs:22`

```rust
pub fn qsl_macro(args: TokenStream, item: TokenStream) -> TokenStream {
```

`qsl/macros/src/lib.rs:26`

```rust
pub fn schem_macro(args: TokenStream, item: TokenStream) -> TokenStream {
```

## qsl/src/analysis.rs

`qsl/src/analysis.rs:5`

```rust
pub enum BitRegister {
```

`qsl/src/analysis.rs:14`

```rust
pub fn for_bits(bits: usize) -> QslResult<Self> {
```

`qsl/src/analysis.rs:25`

```rust
pub fn bits(self) -> usize {
```

`qsl/src/analysis.rs:35`

```rust
pub fn reader(self) -> &'static str {
```

`qsl/src/analysis.rs:45`

```rust
pub fn writer(self) -> &'static str {
```

`qsl/src/analysis.rs:57`

```rust
pub struct BitBlock {
```

`qsl/src/analysis.rs:62`

```rust
pub struct QslIoPlan {
```

`qsl/src/analysis.rs:111`

```rust
pub fn analyze(schema: &QslSchema) -> QslResult<Self> {
```

## qsl/src/bit_io.rs

`qsl/src/bit_io.rs:70`

```rust
pub unsafe fn read_inline_string_into_with<const MAX: usize, H: crate::StringHeader>( &mut self, out: *mut crate::InlineString<MAX>, ) {
```

`qsl/src/bit_io.rs:81`

```rust
pub unsafe fn read_raw_inline_string_into_with<const MAX: usize, H: crate::StringHeader>( &mut self, out: *mut crate::InlineString<MAX>, ) {
```

`qsl/src/bit_io.rs:92`

```rust
pub unsafe fn read_inline_string_with<const MAX: usize, H: crate::StringHeader>( &mut self, ) -> crate::InlineString<MAX> {
```

`qsl/src/bit_io.rs:103`

```rust
pub unsafe fn read_raw_inline_string_with<const MAX: usize, H: crate::StringHeader>( &mut self, ) -> crate::InlineString<MAX> {
```

`qsl/src/bit_io.rs:112`

```rust
pub fn as_ptr(&self) -> *const u8 {
```

`qsl/src/bit_io.rs:118`

```rust
pub unsafe fn read_exact(&mut self, n: usize) -> *const u8 {
```

`qsl/src/bit_io.rs:127`

```rust
pub unsafe fn read_u8(&mut self) -> u8 {
```

`qsl/src/bit_io.rs:133`

```rust
pub unsafe fn read_u16(&mut self) -> u16 {
```

`qsl/src/bit_io.rs:139`

```rust
pub unsafe fn read_u32(&mut self) -> u32 {
```

`qsl/src/bit_io.rs:145`

```rust
pub unsafe fn read_u64(&mut self) -> u64 {
```

`qsl/src/bit_io.rs:151`

```rust
pub unsafe fn read_i8(&mut self) -> i8 {
```

`qsl/src/bit_io.rs:157`

```rust
pub unsafe fn read_i16(&mut self) -> i16 {
```

`qsl/src/bit_io.rs:163`

```rust
pub unsafe fn read_i32(&mut self) -> i32 {
```

`qsl/src/bit_io.rs:169`

```rust
pub unsafe fn read_i64(&mut self) -> i64 {
```

`qsl/src/bit_io.rs:175`

```rust
pub unsafe fn read_inline_string<const MAX: usize>(&mut self) -> crate::InlineString<MAX> {
```

`qsl/src/bit_io.rs:184`

```rust
pub unsafe fn read_inline_string_into<const MAX: usize>( &mut self, out: *mut crate::InlineString<MAX>, ) {
```

`qsl/src/bit_io.rs:195`

```rust
pub unsafe fn read_raw_inline_string_into<const MAX: usize>( &mut self, out: *mut crate::InlineString<MAX>, ) {
```

`qsl/src/bit_io.rs:210`

```rust
pub unsafe fn write_inline_string_with<const MAX: usize, H: crate::StringHeader>( &mut self, value: &crate::InlineString<MAX>, ) {
```

`qsl/src/bit_io.rs:219`

```rust
pub fn as_mut_ptr(&self) -> *mut u8 {
```

`qsl/src/bit_io.rs:225`

```rust
pub unsafe fn claim_bytes(&mut self, n: usize) -> *mut u8 {
```

`qsl/src/bit_io.rs:234`

```rust
pub unsafe fn write_bytes_unchecked(&mut self, bytes: &[u8]) {
```

`qsl/src/bit_io.rs:240`

```rust
pub unsafe fn write_ptr_unchecked(&mut self, source: *const u8, n: usize) {
```

`qsl/src/bit_io.rs:247`

```rust
pub unsafe fn write_zeros_unchecked(&mut self, n: usize) {
```

`qsl/src/bit_io.rs:254`

```rust
pub unsafe fn write_inline_string<const MAX: usize>( &mut self, value: &crate::InlineString<MAX>, ) {
```

`qsl/src/bit_io.rs:264`

```rust
pub struct QslByteReader {
```

`qsl/src/bit_io.rs:271`

```rust
pub unsafe fn new(ptr: *const u8) -> Self {
```

`qsl/src/bit_io.rs:275`

```rust
pub fn align(&mut self) {
```

`qsl/src/bit_io.rs:279`

```rust
pub unsafe fn read_bits(&mut self, width: u32) -> u64 {
```

`qsl/src/bit_io.rs:285`

```rust
pub struct QslByteWriter {
```

`qsl/src/bit_io.rs:292`

```rust
pub unsafe fn from_raw(ptr: *mut u8) -> Self {
```

`qsl/src/bit_io.rs:296`

```rust
pub fn flush_unchecked(&mut self) {
```

`qsl/src/bit_io.rs:300`

```rust
pub unsafe fn write_bits_unchecked(&mut self, _value: u64, width: u32) {
```

`qsl/src/bit_io.rs:307`

```rust
pub struct $reader {
```

`qsl/src/bit_io.rs:316`

```rust
pub unsafe fn new(ptr: *const u8) -> Self {
```

`qsl/src/bit_io.rs:326`

```rust
pub unsafe fn align(&mut self) {
```

`qsl/src/bit_io.rs:333`

```rust
pub unsafe fn read_bits(&mut self, width: u32) -> u64 {
```

`qsl/src/bit_io.rs:348`

```rust
pub struct $writer {
```

`qsl/src/bit_io.rs:357`

```rust
pub unsafe fn from_raw(ptr: *mut u8) -> Self {
```

`qsl/src/bit_io.rs:367`

```rust
pub unsafe fn write_bits_unchecked(&mut self, value: u64, width: u32) {
```

`qsl/src/bit_io.rs:382`

```rust
pub unsafe fn flush_unchecked(&mut self) {
```

## qsl/src/compat.rs

`qsl/src/compat.rs:10`

```rust
pub enum ParserQslType {
```

`qsl/src/compat.rs:25`

```rust
pub struct ParserQslField {
```

`qsl/src/compat.rs:34`

```rust
pub struct ParserQslSchema {
```

`qsl/src/compat.rs:42`

```rust
pub fn new(flags: usize) -> Self {
```

`qsl/src/compat.rs:48`

```rust
pub fn optional_count(&self) -> usize {
```

`qsl/src/compat.rs:51`

```rust
pub fn validate(&self) -> QslResult<()> {
```

## qsl/src/deq.rs

`qsl/src/deq.rs:13`

```rust
pub struct DeqConstant {
```

`qsl/src/deq.rs:19`

```rust
pub struct DeqConfig {
```

`qsl/src/deq.rs:26`

```rust
pub struct GenerationConfig {
```

`qsl/src/deq.rs:46`

```rust
pub fn integer_values(&self) -> HashMap<String, i128> {
```

`qsl/src/deq.rs:54`

```rust
pub fn parse_deq(source: &str) -> Result<DeqConfig, ParseError> {
```

`qsl/src/deq.rs:147`

```rust
pub fn build_qsl_files( qsl_files: &[&str], deq_files: &[&str], out_dir: impl AsRef<Path>, ) -> Result<PathBuf, Box<dyn std::error::Error>> {
```

`qsl/src/deq.rs:155`

```rust
pub fn build_qsl_files_with_rust( qsl_files: &[&str], deq_files: &[&str], rust_files: &[&str], out_dir: impl AsRef<Path>, ) -> Result<PathBuf, Box<dyn std::error::Error>> {
```

## qsl/src/interpreter.rs

`qsl/src/interpreter.rs:20`

```rust
pub fn schema_maximum_size(schema: &QslSchema) -> QslResult<Option<usize>> {
```

`qsl/src/interpreter.rs:132`

```rust
pub trait TypeInterpreter {
```

`qsl/src/interpreter.rs:133`

```rust
fn type_name(&self, out: &mut Vec<u8>, borrowed: bool) -> QslResult<()>;
```

`qsl/src/interpreter.rs:134`

```rust
fn size(&self, out: &mut Vec<u8>, expr: &str, width: usize, compact: bool) -> QslResult<()>;
```

`qsl/src/interpreter.rs:135`

```rust
fn write(&self, out: &mut Vec<u8>, expr: &str) -> QslResult<()>;
```

`qsl/src/interpreter.rs:136`

```rust
fn read(&self, out: &mut Vec<u8>, name: &str, borrowed: bool) -> QslResult<()>;
```

`qsl/src/interpreter.rs:137`

```rust
fn comp_write(&self, out: &mut Vec<u8>, expr: &str, bits: usize) -> QslResult<()>;
```

`qsl/src/interpreter.rs:138`

```rust
fn comp_read( &self, out: &mut Vec<u8>, name: &str, bits: usize, borrowed: bool, ) -> QslResult<()>;
```

`qsl/src/interpreter.rs:2238`

```rust
pub struct QslInterpreter {
```

`qsl/src/interpreter.rs:2266`

```rust
pub fn new() -> Self {
```

`qsl/src/interpreter.rs:2270`

```rust
pub fn generate(&mut self, schemas: &[QslSchema]) -> QslResult<()> {
```

`qsl/src/interpreter.rs:2292`

```rust
pub fn generate_parsed(&mut self, schemas: &[crate::compat::ParserQslSchema]) -> QslResult<()> {
```

`qsl/src/interpreter.rs:2300`

```rust
pub fn generate_field(&mut self, field: &QslField, borrowed: bool) -> QslResult<()> {
```

`qsl/src/interpreter.rs:2683`

```rust
pub fn generate_default(&mut self, schema: &QslSchema) -> QslResult<()> {
```

`qsl/src/interpreter.rs:2694`

```rust
pub fn generate_lifetime(&mut self, schema: &QslSchema) -> QslResult<()> {
```

`qsl/src/interpreter.rs:2708`

```rust
pub fn push(&self) -> std::io::Result<()> {
```

## qsl/src/io.rs

`qsl/src/io.rs:7`

```rust
pub use checked::{QslCheckedBitsReader, QslCheckedBitsWriter, QslCheckedReader, QslCheckedWriter};
```

`qsl/src/io.rs:9`

```rust
pub type QslResult<T> = Result<T, QslError>;
```

`qsl/src/io.rs:12`

```rust
pub enum QslError {
```

`qsl/src/io.rs:36`

```rust
pub struct QslRawWriter {
```

`qsl/src/io.rs:44`

```rust
pub unsafe fn new(ptr: *mut u8) -> Self {
```

`qsl/src/io.rs:51`

```rust
pub fn from(out: &mut [u8]) -> Self {
```

`qsl/src/io.rs:61`

```rust
pub unsafe fn write_u8(&mut self, value: u8) {
```

`qsl/src/io.rs:69`

```rust
pub unsafe fn write_u16(&mut self, value: u16) {
```

`qsl/src/io.rs:77`

```rust
pub unsafe fn write_u32(&mut self, value: u32) {
```

`qsl/src/io.rs:85`

```rust
pub unsafe fn write_u64(&mut self, value: u64) {
```

`qsl/src/io.rs:93`

```rust
pub unsafe fn write(&mut self, source: *const u8, len: usize) {
```

`qsl/src/io.rs:101`

```rust
pub struct QslRawReader {
```

`qsl/src/io.rs:109`

```rust
pub unsafe fn new(ptr: *const u8) -> Self {
```

`qsl/src/io.rs:115`

```rust
pub fn from(input: &[u8]) -> Self {
```

`qsl/src/io.rs:124`

```rust
pub unsafe fn read_u8(&mut self) -> u8 {
```

`qsl/src/io.rs:133`

```rust
pub unsafe fn read_u16(&mut self) -> u16 {
```

`qsl/src/io.rs:142`

```rust
pub unsafe fn read_u32(&mut self) -> u32 {
```

`qsl/src/io.rs:151`

```rust
pub unsafe fn read_u64(&mut self) -> u64 {
```

`qsl/src/io.rs:159`

```rust
pub unsafe fn take(&mut self, len: usize) -> *const u8 {
```

`qsl/src/io.rs:169`

```rust
pub struct QslFastBitsReader {
```

`qsl/src/io.rs:180`

```rust
pub unsafe fn read_inline_string_into_with<const MAX: usize, H: crate::StringHeader>( &mut self, out: *mut crate::InlineString<MAX>, ) {
```

`qsl/src/io.rs:191`

```rust
pub unsafe fn read_raw_inline_string_into_with<const MAX: usize, H: crate::StringHeader>( &mut self, out: *mut crate::InlineString<MAX>, ) {
```

`qsl/src/io.rs:202`

```rust
pub unsafe fn read_inline_string_with<const MAX: usize, H: crate::StringHeader>( &mut self, ) -> crate::InlineString<MAX> {
```

`qsl/src/io.rs:213`

```rust
pub unsafe fn read_raw_inline_string_with<const MAX: usize, H: crate::StringHeader>( &mut self, ) -> crate::InlineString<MAX> {
```

`qsl/src/io.rs:224`

```rust
pub fn as_ptr(&self) -> *const u8 {
```

`qsl/src/io.rs:232`

```rust
pub unsafe fn read_raw_inline_string_into<const MAX: usize>( &mut self, output: *mut crate::InlineString<MAX>, ) {
```

`qsl/src/io.rs:244`

```rust
pub unsafe fn read_inline_string_into<const MAX: usize>( &mut self, output: *mut crate::InlineString<MAX>, ) {
```

`qsl/src/io.rs:256`

```rust
pub unsafe fn read_inline_string<const MAX: usize>(&mut self) -> crate::InlineString<MAX> {
```

`qsl/src/io.rs:267`

```rust
pub unsafe fn new(ptr: *const u8) -> Self {
```

`qsl/src/io.rs:278`

```rust
pub fn from(input: &[u8]) -> Self {
```

`qsl/src/io.rs:292`

```rust
pub unsafe fn read_bits(&mut self, width: u32) -> u64 {
```

`qsl/src/io.rs:311`

```rust
pub unsafe fn align(&mut self) {
```

`qsl/src/io.rs:320`

```rust
pub unsafe fn read_exact(&mut self, len: usize) -> *const u8 {
```

`qsl/src/io.rs:332`

```rust
pub unsafe fn read_u8(&mut self) -> u8 {
```

`qsl/src/io.rs:343`

```rust
pub unsafe fn read_u16(&mut self) -> u16 {
```

`qsl/src/io.rs:354`

```rust
pub unsafe fn read_u32(&mut self) -> u32 {
```

`qsl/src/io.rs:365`

```rust
pub unsafe fn read_u64(&mut self) -> u64 {
```

`qsl/src/io.rs:375`

```rust
pub unsafe fn read_i8(&mut self) -> i8 {
```

`qsl/src/io.rs:382`

```rust
pub unsafe fn read_i16(&mut self) -> i16 {
```

`qsl/src/io.rs:389`

```rust
pub unsafe fn read_i32(&mut self) -> i32 {
```

`qsl/src/io.rs:396`

```rust
pub unsafe fn read_i64(&mut self) -> i64 {
```

`qsl/src/io.rs:408`

```rust
pub struct QslUncheckedWriter {
```

`qsl/src/io.rs:418`

```rust
pub unsafe fn write_inline_string_with<const MAX: usize, H: crate::StringHeader>( &mut self, value: &crate::InlineString<MAX>, ) {
```

`qsl/src/io.rs:429`

```rust
pub fn as_mut_ptr(&self) -> *mut u8 {
```

`qsl/src/io.rs:436`

```rust
pub unsafe fn write_inline_string<const MAX: usize>( &mut self, value: &crate::InlineString<MAX>, ) {
```

`qsl/src/io.rs:447`

```rust
pub unsafe fn from_raw(ptr: *mut u8) -> Self {
```

`qsl/src/io.rs:458`

```rust
pub fn from(out: &mut [u8]) -> Self {
```

`qsl/src/io.rs:470`

```rust
pub unsafe fn write_bytes_unchecked(&mut self, bytes: &[u8]) {
```

`qsl/src/io.rs:480`

```rust
pub unsafe fn write_ptr_unchecked(&mut self, source: *const u8, len: usize) {
```

`qsl/src/io.rs:490`

```rust
pub unsafe fn write_zeros_unchecked(&mut self, count: usize) {
```

`qsl/src/io.rs:500`

```rust
pub unsafe fn write_bits_unchecked(&mut self, value: u64, width: u32) {
```

`qsl/src/io.rs:517`

```rust
pub unsafe fn flush_unchecked(&mut self) {
```

`qsl/src/io.rs:532`

```rust
pub unsafe fn claim_bytes(&mut self, n: usize) -> *mut u8 {
```

`qsl/src/io.rs:543`

```rust
pub unsafe fn write_bytes(&mut self, bytes: &[u8]) {
```

`qsl/src/io.rs:551`

```rust
pub unsafe fn write_zeros(&mut self, n: usize) {
```

`qsl/src/io.rs:559`

```rust
pub unsafe fn write_bits(&mut self, v: u64, width: u32) {
```

`qsl/src/io.rs:568`

```rust
pub unsafe fn flush(&mut self) {
```

`qsl/src/io.rs:575`

```rust
pub unsafe fn write_u8(&mut self, value: u8) {
```

`qsl/src/io.rs:585`

```rust
pub unsafe fn write_u16(&mut self, value: u16) {
```

`qsl/src/io.rs:595`

```rust
pub unsafe fn write_u32(&mut self, value: u32) {
```

`qsl/src/io.rs:605`

```rust
pub unsafe fn write_u64(&mut self, value: u64) {
```

`qsl/src/io.rs:614`

```rust
pub unsafe fn write_i8(&mut self, v: i8) {
```

`qsl/src/io.rs:621`

```rust
pub unsafe fn write_i16(&mut self, v: i16) {
```

`qsl/src/io.rs:628`

```rust
pub unsafe fn write_i32(&mut self, v: i32) {
```

`qsl/src/io.rs:635`

```rust
pub unsafe fn write_i64(&mut self, v: i64) {
```

`qsl/src/io.rs:642`

```rust
pub type QslWriter = QslRawWriter;
```

`qsl/src/io.rs:643`

```rust
pub type QslReader = QslRawReader;
```

`qsl/src/io.rs:644`

```rust
pub type QslBitsWriter = QslUncheckedWriter;
```

`qsl/src/io.rs:645`

```rust
pub type QslBitsReader = QslFastBitsReader;
```

`qsl/src/io.rs:649`

```rust
pub struct QslSizer {
```

`qsl/src/io.rs:656`

```rust
pub fn bits(&mut self, n: usize) -> QslResult<()> {
```

`qsl/src/io.rs:666`

```rust
pub fn align(&mut self) -> QslResult<()> {
```

`qsl/src/io.rs:674`

```rust
pub fn bytes(&mut self, n: usize) -> QslResult<()> {
```

`qsl/src/io.rs:680`

```rust
pub fn finish(mut self) -> QslResult<usize> {
```

## qsl/src/io_checked.rs

`qsl/src/io_checked.rs:9`

```rust
pub struct QslCheckedWriter<'a> {
```

`qsl/src/io_checked.rs:17`

```rust
pub fn from(out: &'a mut [u8]) -> Self {
```

`qsl/src/io_checked.rs:22`

```rust
pub unsafe fn from_raw_parts(ptr: *mut u8, len: usize) -> Self {
```

`qsl/src/io_checked.rs:31`

```rust
pub fn from_raw(out: &'a mut crate::serialize::SerOut) -> Self {
```

`qsl/src/io_checked.rs:34`

```rust
pub fn position(&self) -> usize {
```

`qsl/src/io_checked.rs:37`

```rust
pub fn write(&mut self, bytes: &[u8]) -> QslResult<()> {
```

`qsl/src/io_checked.rs:53`

```rust
pub fn write_zeros(&mut self, len: usize) -> QslResult<()> {
```

`qsl/src/io_checked.rs:66`

```rust
pub fn write_u8(&mut self, v: u8) -> QslResult<()> {
```

`qsl/src/io_checked.rs:69`

```rust
pub fn write_u16(&mut self, v: u16) -> QslResult<()> {
```

`qsl/src/io_checked.rs:72`

```rust
pub fn write_u32(&mut self, v: u32) -> QslResult<()> {
```

`qsl/src/io_checked.rs:75`

```rust
pub fn write_u64(&mut self, v: u64) -> QslResult<()> {
```

`qsl/src/io_checked.rs:78`

```rust
pub fn write_i8(&mut self, v: i8) -> QslResult<()> {
```

`qsl/src/io_checked.rs:81`

```rust
pub fn write_i16(&mut self, v: i16) -> QslResult<()> {
```

`qsl/src/io_checked.rs:84`

```rust
pub fn write_i32(&mut self, v: i32) -> QslResult<()> {
```

`qsl/src/io_checked.rs:87`

```rust
pub fn write_i64(&mut self, v: i64) -> QslResult<()> {
```

`qsl/src/io_checked.rs:92`

```rust
pub struct QslCheckedReader<'a> {
```

`qsl/src/io_checked.rs:99`

```rust
pub fn new(bytes: &'a [u8]) -> Self {
```

`qsl/src/io_checked.rs:104`

```rust
pub unsafe fn from_raw_parts(ptr: *const u8, len: usize) -> Self {
```

`qsl/src/io_checked.rs:112`

```rust
pub fn position(&self) -> usize {
```

`qsl/src/io_checked.rs:115`

```rust
pub fn read_bytes(&mut self, n: usize) -> QslResult<&'a [u8]> {
```

`qsl/src/io_checked.rs:124`

```rust
pub fn read_u8(&mut self) -> QslResult<u8> {
```

`qsl/src/io_checked.rs:127`

```rust
pub fn read_u16(&mut self) -> QslResult<u16> {
```

`qsl/src/io_checked.rs:130`

```rust
pub fn read_u32(&mut self) -> QslResult<u32> {
```

`qsl/src/io_checked.rs:133`

```rust
pub fn read_u64(&mut self) -> QslResult<u64> {
```

`qsl/src/io_checked.rs:136`

```rust
pub fn read_i8(&mut self) -> QslResult<i8> {
```

`qsl/src/io_checked.rs:139`

```rust
pub fn read_i16(&mut self) -> QslResult<i16> {
```

`qsl/src/io_checked.rs:142`

```rust
pub fn read_i32(&mut self) -> QslResult<i32> {
```

`qsl/src/io_checked.rs:145`

```rust
pub fn read_i64(&mut self) -> QslResult<i64> {
```

`qsl/src/io_checked.rs:152`

```rust
pub struct QslCheckedBitsWriter<'a> {
```

`qsl/src/io_checked.rs:161`

```rust
pub fn from(out: &'a mut [u8]) -> Self {
```

`qsl/src/io_checked.rs:166`

```rust
pub unsafe fn from_raw_parts(ptr: *mut u8, len: usize) -> Self {
```

`qsl/src/io_checked.rs:177`

```rust
pub fn from_raw(out: &'a mut crate::serialize::SerOut) -> Self {
```

`qsl/src/io_checked.rs:180`

```rust
pub fn position(&self) -> usize {
```

`qsl/src/io_checked.rs:193`

```rust
pub fn write_bits(&mut self, mut value: u64, mut width: u32) -> QslResult<()> {
```

`qsl/src/io_checked.rs:215`

```rust
pub fn flush(&mut self) -> QslResult<()> {
```

`qsl/src/io_checked.rs:223`

```rust
pub fn write_bytes(&mut self, bytes: &[u8]) -> QslResult<()> {
```

`qsl/src/io_checked.rs:240`

```rust
pub fn write_zeros(&mut self, n: usize) -> QslResult<()> {
```

`qsl/src/io_checked.rs:254`

```rust
pub fn write_u8(&mut self, v: u8) -> QslResult<()> {
```

`qsl/src/io_checked.rs:257`

```rust
pub fn write_u16(&mut self, v: u16) -> QslResult<()> {
```

`qsl/src/io_checked.rs:260`

```rust
pub fn write_u32(&mut self, v: u32) -> QslResult<()> {
```

`qsl/src/io_checked.rs:263`

```rust
pub fn write_u64(&mut self, v: u64) -> QslResult<()> {
```

`qsl/src/io_checked.rs:266`

```rust
pub fn write_i8(&mut self, v: i8) -> QslResult<()> {
```

`qsl/src/io_checked.rs:269`

```rust
pub fn write_i16(&mut self, v: i16) -> QslResult<()> {
```

`qsl/src/io_checked.rs:272`

```rust
pub fn write_i32(&mut self, v: i32) -> QslResult<()> {
```

`qsl/src/io_checked.rs:275`

```rust
pub fn write_i64(&mut self, v: i64) -> QslResult<()> {
```

`qsl/src/io_checked.rs:280`

```rust
pub struct QslCheckedBitsReader<'a> {
```

`qsl/src/io_checked.rs:289`

```rust
pub fn new(input: &'a [u8]) -> Self {
```

`qsl/src/io_checked.rs:294`

```rust
pub unsafe fn from_raw_parts(ptr: *const u8, len: usize) -> Self {
```

`qsl/src/io_checked.rs:304`

```rust
pub fn position(&self) -> usize {
```

`qsl/src/io_checked.rs:307`

```rust
pub fn read_bits(&mut self, width: u32) -> QslResult<u64> {
```

`qsl/src/io_checked.rs:332`

```rust
pub fn align(&mut self) -> QslResult<()> {
```

`qsl/src/io_checked.rs:340`

```rust
pub fn read_bytes(&mut self, n: usize) -> QslResult<&'a [u8]> {
```

`qsl/src/io_checked.rs:350`

```rust
pub fn read_u8(&mut self) -> QslResult<u8> {
```

`qsl/src/io_checked.rs:353`

```rust
pub fn read_u16(&mut self) -> QslResult<u16> {
```

`qsl/src/io_checked.rs:356`

```rust
pub fn read_u32(&mut self) -> QslResult<u32> {
```

`qsl/src/io_checked.rs:359`

```rust
pub fn read_u64(&mut self) -> QslResult<u64> {
```

`qsl/src/io_checked.rs:362`

```rust
pub fn read_i8(&mut self) -> QslResult<i8> {
```

`qsl/src/io_checked.rs:365`

```rust
pub fn read_i16(&mut self) -> QslResult<i16> {
```

`qsl/src/io_checked.rs:368`

```rust
pub fn read_i32(&mut self) -> QslResult<i32> {
```

`qsl/src/io_checked.rs:371`

```rust
pub fn read_i64(&mut self) -> QslResult<i64> {
```

## qsl/src/length_graph.rs

`qsl/src/length_graph.rs:6`

```rust
pub struct Domain {
```

`qsl/src/length_graph.rs:13`

```rust
pub struct LengthNode {
```

`qsl/src/length_graph.rs:22`

```rust
pub struct OffsetRecipe {
```

`qsl/src/length_graph.rs:29`

```rust
pub struct LengthGraph {
```

`qsl/src/length_graph.rs:37`

```rust
pub fn analyze(s: &QslSchema) -> QslResult<Option<Self>> {
```

`qsl/src/length_graph.rs:219`

```rust
pub fn event_index(&self, len: usize) -> Option<usize> {
```

## qsl/src/lib.rs

`qsl/src/lib.rs:2`

```rust
pub mod compat;
```

`qsl/src/lib.rs:3`

```rust
pub mod deq;
```

`qsl/src/lib.rs:4`

```rust
pub mod interpreter;
```

`qsl/src/lib.rs:5`

```rust
pub mod io;
```

`qsl/src/lib.rs:6`

```rust
pub mod parser;
```

`qsl/src/lib.rs:7`

```rust
pub mod parser_util;
```

`qsl/src/lib.rs:8`

```rust
pub mod serialize;
```

`qsl/src/lib.rs:9`

```rust
pub mod strings;
```

`qsl/src/lib.rs:10`

```rust
pub mod types;
```

`qsl/src/lib.rs:12`

```rust
pub use compat::*;
```

`qsl/src/lib.rs:13`

```rust
pub use deq::{build_qsl_files, parse_deq, DeqConfig, DeqConstant};
```

`qsl/src/lib.rs:14`

```rust
pub use interpreter::*;
```

`qsl/src/lib.rs:15`

```rust
pub use io::*;
```

`qsl/src/lib.rs:16`

```rust
pub use parser::{parse_qsl, parse_qsl_with_constants, ParseError};
```

`qsl/src/lib.rs:17`

```rust
pub use serialize::*;
```

`qsl/src/lib.rs:18`

```rust
pub use strings::*;
```

`qsl/src/lib.rs:19`

```rust
pub use types::*;
```

`qsl/src/lib.rs:21`

```rust
pub mod optional;
```

`qsl/src/lib.rs:22`

```rust
pub use optional::OptionalOffsets;
```

`qsl/src/lib.rs:24`

```rust
pub mod socket;
```

`qsl/src/lib.rs:25`

```rust
pub use socket::RawSocket;
```

`qsl/src/lib.rs:27`

```rust
pub mod analysis;
```

`qsl/src/lib.rs:28`

```rust
pub mod bit_io;
```

`qsl/src/lib.rs:29`

```rust
pub mod payload_quant;
```

`qsl/src/lib.rs:30`

```rust
pub mod qps;
```

`qsl/src/lib.rs:31`

```rust
pub mod qps_bytes;
```

`qsl/src/lib.rs:32`

```rust
pub mod static_index;
```

`qsl/src/lib.rs:33`

```rust
pub use payload_quant::{QuantizedFrame, QuantizedLease, QuantizedOut, QuantizedRegions};
```

`qsl/src/lib.rs:34`

```rust
pub mod rt;
```

`qsl/src/lib.rs:36`

```rust
pub mod rust_macro;
```

`qsl/src/lib.rs:37`

```rust
pub mod schema_macro;
```

`qsl/src/lib.rs:39`

```rust
pub use deq::build_qsl_files_with_rust;
```

`qsl/src/lib.rs:40`

```rust
pub use qsl_macros_impl::{qsl_macro, schem_macro};
```

`qsl/src/lib.rs:41`

```rust
pub mod length_graph;
```

`qsl/src/lib.rs:42`

```rust
pub mod static_optional;
```

`qsl/src/lib.rs:43`

```rust
pub mod validation;
```

`qsl/src/lib.rs:44`

```rust
pub use analysis::*;
```

`qsl/src/lib.rs:45`

```rust
pub use bit_io::*;
```

`qsl/src/lib.rs:49`

```rust
pub mod native;
```

`qsl/src/lib.rs:51`

```rust
pub mod schema_hash;
```

`qsl/src/lib.rs:52`

```rust
pub use schema_hash::{ SchemaHashAlgorithm, SchemaHashKind, SchemaHashes, SchemaIdentity, SCHEMA_HASH_VERSION, };
```

## qsl/src/native/memory.rs

`qsl/src/native/memory.rs:56`

```rust
pub struct NativeRegion {
```

`qsl/src/native/memory.rs:63`

```rust
pub fn new(capacity: usize) -> io::Result<Self> {
```

`qsl/src/native/memory.rs:83`

```rust
pub fn capacity(&self) -> usize {
```

`qsl/src/native/memory.rs:86`

```rust
pub fn used(&self) -> usize {
```

`qsl/src/native/memory.rs:90`

```rust
pub fn reset(&mut self) {
```

`qsl/src/native/memory.rs:95`

```rust
pub fn prefault(&mut self) {
```

`qsl/src/native/memory.rs:120`

```rust
pub fn buffer(&self, capacity: usize, alignment: usize) -> io::Result<RegionBuffer<'_>> {
```

`qsl/src/native/memory.rs:131`

```rust
pub fn slot<T>(&self) -> io::Result<RegionSlot<'_, T>> {
```

`qsl/src/native/memory.rs:146`

```rust
pub struct RegionBuffer<'r> {
```

`qsl/src/native/memory.rs:154`

```rust
pub fn capacity(&self) -> usize {
```

`qsl/src/native/memory.rs:158`

```rust
pub fn clear(&mut self) {
```

`qsl/src/native/memory.rs:162`

```rust
pub fn as_mut_ptr(&mut self) -> *mut u8 {
```

`qsl/src/native/memory.rs:165`

```rust
pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
```

`qsl/src/native/memory.rs:176`

```rust
pub unsafe fn set_len(&mut self, len: usize) {
```

`qsl/src/native/memory.rs:180`

```rust
pub fn write_from_slice(&mut self, bytes: &[u8]) -> io::Result<()> {
```

`qsl/src/native/memory.rs:205`

```rust
pub struct RegionSlot<'r, T> {
```

`qsl/src/native/memory.rs:211`

```rust
pub fn as_mut_ptr(&mut self) -> *mut T {
```

`qsl/src/native/memory.rs:214`

```rust
pub fn write(self, value: T) -> RegionValue<'r, T> {
```

`qsl/src/native/memory.rs:223`

```rust
pub unsafe fn assume_init(self) -> RegionValue<'r, T> {
```

`qsl/src/native/memory.rs:230`

```rust
pub struct RegionValue<'r, T> {
```

`qsl/src/native/memory.rs:341`

```rust
pub struct SharedRegion {
```

`qsl/src/native/memory.rs:345`

```rust
pub fn create(name: &str, capacity: usize) -> io::Result<Self> {
```

`qsl/src/native/memory.rs:394`

```rust
pub unsafe fn open(name: &str) -> io::Result<Self> {
```

`qsl/src/native/memory.rs:425`

```rust
pub fn capacity(&self) -> usize {
```

`qsl/src/native/memory.rs:428`

```rust
pub fn used(&self) -> usize {
```

`qsl/src/native/memory.rs:433`

```rust
pub fn reserve(&self, capacity: usize, alignment: usize) -> io::Result<SharedWrite> {
```

`qsl/src/native/memory.rs:477`

```rust
pub unsafe fn read_published(&self, offset: u64) -> io::Result<Option<SharedBytes>> {
```

`qsl/src/native/memory.rs:521`

```rust
pub struct SharedWrite {
```

`qsl/src/native/memory.rs:531`

```rust
pub fn offset(&self) -> u64 {
```

`qsl/src/native/memory.rs:535`

```rust
pub fn capacity(&self) -> usize {
```

`qsl/src/native/memory.rs:539`

```rust
pub fn as_mut_ptr(&mut self) -> *mut u8 {
```

`qsl/src/native/memory.rs:542`

```rust
pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
```

`qsl/src/native/memory.rs:552`

```rust
pub unsafe fn set_len(&mut self, len: usize) {
```

`qsl/src/native/memory.rs:556`

```rust
pub fn write_from_slice(&mut self, bytes: &[u8]) -> io::Result<()> {
```

`qsl/src/native/memory.rs:566`

```rust
pub fn publish(mut self) -> SharedBytes {
```

`qsl/src/native/memory.rs:609`

```rust
pub struct SharedBytes {
```

`qsl/src/native/memory.rs:617`

```rust
pub fn offset(&self) -> u64 {
```

## qsl/src/native/mod.rs

`qsl/src/native/mod.rs:6`

```rust
pub use memory::{ NativeRegion, RegionBuffer, RegionSlot, RegionValue, SharedBytes, SharedRegion, SharedWrite, };
```

`qsl/src/native/mod.rs:9`

```rust
pub use socket::{NativeListener, NativeSocket};
```

## qsl/src/native/socket.rs

`qsl/src/native/socket.rs:134`

```rust
pub struct NativeSocket {
```

`qsl/src/native/socket.rs:140`

```rust
pub fn connect(address: SocketAddr, max_frame: usize) -> io::Result<Self> {
```

`qsl/src/native/socket.rs:169`

```rust
pub fn set_read_timeout(&self, timeout: Option<std::time::Duration>) -> io::Result<()> {
```

`qsl/src/native/socket.rs:173`

```rust
pub fn set_write_timeout(&self, timeout: Option<std::time::Duration>) -> io::Result<()> {
```

`qsl/src/native/socket.rs:198`

```rust
pub fn local_addr(&self) -> io::Result<SocketAddr> {
```

`qsl/src/native/socket.rs:220`

```rust
pub unsafe fn receive_raw(&mut self, ptr: *mut u8, capacity: usize) -> io::Result<usize> {
```

`qsl/src/native/socket.rs:241`

```rust
pub fn send_raw(&mut self, bytes: &[u8]) -> io::Result<usize> {
```

`qsl/src/native/socket.rs:304`

```rust
pub fn receive_into(&mut self, buffer: &mut RegionBuffer<'_>) -> io::Result<usize> {
```

`qsl/src/native/socket.rs:313`

```rust
pub fn receive_shared(&mut self, buffer: &mut SharedWrite) -> io::Result<usize> {
```

`qsl/src/native/socket.rs:325`

```rust
pub fn send_frame(&mut self, payload: &[u8]) -> io::Result<()> {
```

`qsl/src/native/socket.rs:390`

```rust
pub unsafe fn send_record<T: Serializable>( &mut self, value: &T, buffer: &mut RegionBuffer<'_>, ) -> io::Result<usize> {
```

`qsl/src/native/socket.rs:409`

```rust
pub unsafe fn send_borrowed_record<'a, T: BorrowSerializable<'a>>( &mut self, value: &T, buffer: &mut RegionBuffer<'_>, ) -> io::Result<usize> {
```

`qsl/src/native/socket.rs:427`

```rust
pub struct NativeListener {
```

`qsl/src/native/socket.rs:432`

```rust
pub fn bind(address: SocketAddr, max_frame: usize, backlog: i32) -> io::Result<Self> {
```

`qsl/src/native/socket.rs:460`

```rust
pub fn local_addr(&self) -> io::Result<SocketAddr> {
```

`qsl/src/native/socket.rs:463`

```rust
pub fn accept(&self) -> io::Result<NativeSocket> {
```

## qsl/src/optional.rs

`qsl/src/optional.rs:8`

```rust
pub struct OptionalOffsets<'a> {
```

`qsl/src/optional.rs:14`

```rust
pub fn parse(table_and_payload: &'a [u8]) -> QslResult<Self> {
```

`qsl/src/optional.rs:45`

```rust
pub fn len(&self) -> usize {
```

`qsl/src/optional.rs:49`

```rust
pub fn is_empty(&self) -> bool {
```

`qsl/src/optional.rs:54`

```rust
pub fn offset(&self, index: usize) -> Option<usize> {
```

`qsl/src/optional.rs:68`

```rust
pub fn field_tail(&self, index: usize) -> Option<&'a [u8]> {
```

## qsl/src/parser.rs

`qsl/src/parser.rs:13`

```rust
pub struct ParseError {
```

`qsl/src/parser.rs:27`

```rust
pub fn span(&self) -> (usize, usize) {
```

`qsl/src/parser.rs:244`

```rust
pub fn parse_qsl(source: &str) -> Result<Vec<QslSchema>, ParseError> {
```

`qsl/src/parser.rs:248`

```rust
pub fn parse_qsl_with_constants( source: &str, globals: &HashMap<String, i128>, ) -> Result<Vec<QslSchema>, ParseError> {
```

## qsl/src/parser_util.rs

`qsl/src/parser_util.rs:4`

```rust
pub struct ParseNum(pub usize, pub bool);
```

`qsl/src/parser_util.rs:7`

```rust
pub fn rank_bits(min: ParseNum, max: ParseNum) -> Option<usize> {
```

`qsl/src/parser_util.rs:19`

```rust
pub fn parse_bytes(bytes: &[u8]) -> Option<usize> {
```

`qsl/src/parser_util.rs:33`

```rust
pub fn parse_hex_bytes(bytes: &[u8]) -> Result<usize, &'static str> {
```

`qsl/src/parser_util.rs:55`

```rust
pub fn scan_schema_blocks(source: &[u8]) -> Result<Vec<&[u8]>, &'static str> {
```

`qsl/src/parser_util.rs:74`

```rust
pub fn parse_schema_flags(source: &[u8]) -> Result<usize, &'static str> {
```

`qsl/src/parser_util.rs:96`

```rust
pub fn parse_primitive_type(token: &[u8]) -> Option<crate::compat::ParserQslType> {
```

`qsl/src/parser_util.rs:112`

```rust
pub fn encode_signed_minimum(value: ParseNum) -> usize {
```

## qsl/src/payload_quant.rs

`qsl/src/payload_quant.rs:11`

```rust
pub fn selected_address<T>(mut value: *const T) -> *const T {
```

`qsl/src/payload_quant.rs:26`

```rust
pub fn scalar_operand(mut value: u64) -> u64 {
```

`qsl/src/payload_quant.rs:39`

```rust
pub struct QuantField {
```

`qsl/src/payload_quant.rs:51`

```rust
pub struct PayloadQuantPlan {
```

`qsl/src/payload_quant.rs:63`

```rust
pub fn analyze(s: &QslSchema) -> QslResult<Self> {
```

`qsl/src/payload_quant.rs:177`

```rust
pub unsafe trait QuantizedRecord: Serializable {
```

`qsl/src/payload_quant.rs:182`

```rust
fn quant_validate_values(&self) -> QslResult<()>;
```

`qsl/src/payload_quant.rs:186`

```rust
unsafe fn quant_write_prepared(&self, out: *mut u8) -> usize;
```

`qsl/src/payload_quant.rs:190`

```rust
unsafe fn quant_read_padded(bytes: *const u8, len: usize, out: *mut Self);
```

`qsl/src/payload_quant.rs:192`

```rust
pub trait CheckedQuantizedRecord: QuantizedRecord {
```

`qsl/src/payload_quant.rs:193`

```rust
fn quant_validate(input: &[u8]) -> QslResult<()>;
```

`qsl/src/payload_quant.rs:196`

```rust
pub struct QuantizedFrame<'a, T: QuantizedRecord> {
```

`qsl/src/payload_quant.rs:202`

```rust
pub fn try_new(storage: &'a [u8], len: usize) -> QslResult<Self> {
```

`qsl/src/payload_quant.rs:219`

```rust
pub unsafe fn from_validated(storage: &[u8], len: usize) -> QuantizedFrame<'_, T> {
```

`qsl/src/payload_quant.rs:226`

```rust
pub fn as_slice(&self) -> &[u8] {
```

`qsl/src/payload_quant.rs:229`

```rust
pub fn decode_into<'out>(&self, out: &'out mut std::mem::MaybeUninit<T>) -> &'out mut T {
```

`qsl/src/payload_quant.rs:235`

```rust
pub fn decode_boxed(&self) -> Box<T> {
```

`qsl/src/payload_quant.rs:244`

```rust
pub struct QuantizedOut<T: QuantizedRecord> {
```

`qsl/src/payload_quant.rs:250`

```rust
pub fn encode(value: &T) -> QslResult<Self> {
```

`qsl/src/payload_quant.rs:266`

```rust
pub fn as_slice(&self) -> &[u8] {
```

`qsl/src/payload_quant.rs:269`

```rust
pub fn frame(&self) -> QuantizedFrame<'_, T> {
```

`qsl/src/payload_quant.rs:289`

```rust
pub struct QuantizedRegions<T: QuantizedRecord, const CAP: usize, const N: usize> {
```

`qsl/src/payload_quant.rs:299`

```rust
pub const fn new() -> Self {
```

`qsl/src/payload_quant.rs:305`

```rust
pub fn try_encode(&'static self, value: &T) -> QslResult<QuantizedLease<T, CAP, N>> {
```

`qsl/src/payload_quant.rs:327`

```rust
pub struct QuantizedLease<T: QuantizedRecord, const CAP: usize, const N: usize> {
```

`qsl/src/payload_quant.rs:333`

```rust
pub fn as_slice(&self) -> &[u8] {
```

`qsl/src/payload_quant.rs:336`

```rust
pub fn frame(&self) -> QuantizedFrame<'_, T> {
```

`qsl/src/payload_quant.rs:360`

```rust
pub trait QuantizedDestination<T: QuantizedRecord> {
```

`qsl/src/payload_quant.rs:364`

```rust
unsafe fn decode_padded(self, bytes: *const u8, len: usize);
```

## qsl/src/qps.rs

`qsl/src/qps.rs:9`

```rust
pub use crate::payload_quant::{ PayloadQuantPlan as QpsPlan, QuantField as QpsField, QuantizedFrame, QuantizedLease, QuantizedOut, QuantizedRecord, QuantizedRegions, };
```

`qsl/src/qps.rs:15`

```rust
pub const QPS_CODEC_REVISION: u32 = 2;
```

`qsl/src/qps.rs:17`

```rust
pub use crate::qps_bytes::{ ByteFrame, ByteMode, ByteQps, CheckedBorrowedRecord, CheckedRecord, BYTE_FRAME_HEADER, BYTE_QPS_CODEC_REVISION, };
```

## qsl/src/qps_bytes.rs

`qsl/src/qps_bytes.rs:7`

```rust
pub const BYTE_FRAME_HEADER: usize = 16;
```

`qsl/src/qps_bytes.rs:8`

```rust
pub const BYTE_QPS_CODEC_REVISION: u32 = 3;
```

`qsl/src/qps_bytes.rs:245`

```rust
pub enum ByteMode {
```

`qsl/src/qps_bytes.rs:251`

```rust
pub struct ByteQps<const Q: usize>;
```

`qsl/src/qps_bytes.rs:252`

```rust
pub struct ByteFrame<'a, const Q: usize> {
```

`qsl/src/qps_bytes.rs:258`

```rust
pub fn as_slice(&self) -> &[u8] {
```

`qsl/src/qps_bytes.rs:261`

```rust
pub fn original_len(&self) -> usize {
```

`qsl/src/qps_bytes.rs:264`

```rust
pub fn mode(&self) -> ByteMode {
```

`qsl/src/qps_bytes.rs:267`

```rust
pub fn decode_to<'out>(&self, out: &'out mut [u8]) -> QslResult<&'out mut [u8]> {
```

`qsl/src/qps_bytes.rs:270`

```rust
pub fn decode_record_into<'out, T: CheckedRecord>( &self, restoration: &mut [u8], out: &'out mut MaybeUninit<T>, ) -> QslResult<&'out mut T> {
```

`qsl/src/qps_bytes.rs:283`

```rust
pub unsafe trait CheckedRecord: Serializable {
```

`qsl/src/qps_bytes.rs:285`

```rust
fn qps_validate_values(&self) -> QslResult<()>;
```

`qsl/src/qps_bytes.rs:286`

```rust
fn qps_validate_wire(input: &[u8]) -> QslResult<()>;
```

`qsl/src/qps_bytes.rs:291`

```rust
pub unsafe trait CheckedBorrowedRecord<'a>: BorrowSerializable<'a> {
```

`qsl/src/qps_bytes.rs:293`

```rust
fn qps_validate_values(&self) -> QslResult<()>;
```

`qsl/src/qps_bytes.rs:294`

```rust
fn qps_validate_wire(input: &[u8]) -> QslResult<()>;
```

`qsl/src/qps_bytes.rs:297`

```rust
pub fn max_encoded_len(len: usize) -> QslResult<usize> {
```

`qsl/src/qps_bytes.rs:302`

```rust
pub fn inspect(input: &[u8]) -> QslResult<(ByteMode, usize)> {
```

`qsl/src/qps_bytes.rs:331`

```rust
pub fn encode_to<'out>(input: &[u8], out: &'out mut [u8]) -> QslResult<ByteFrame<'out, Q>> {
```

`qsl/src/qps_bytes.rs:402`

```rust
pub fn decode_to<'out>(input: &[u8], out: &'out mut [u8]) -> QslResult<&'out mut [u8]> {
```

`qsl/src/qps_bytes.rs:407`

```rust
pub fn decode_to_with_limit<'out>( input: &[u8], out: &'out mut [u8], maximum: usize, ) -> QslResult<&'out mut [u8]> {
```

`qsl/src/qps_bytes.rs:471`

```rust
pub fn encode_borrowed_to<'out, 'a, T: CheckedBorrowedRecord<'a>>( value: &T, scratch: &mut [u8], out: &'out mut [u8], ) -> QslResult<ByteFrame<'out, Q>> {
```

`qsl/src/qps_bytes.rs:485`

```rust
pub fn decode_borrowed<'a, T: CheckedBorrowedRecord<'a>>( input: &[u8], restoration: &'a mut [u8], ) -> QslResult<T> {
```

`qsl/src/qps_bytes.rs:493`

```rust
pub fn encode_record_to<'out, T: CheckedRecord>( value: &T, scratch: &mut [u8], out: &'out mut [u8], ) -> QslResult<ByteFrame<'out, Q>> {
```

`qsl/src/qps_bytes.rs:507`

```rust
pub fn decode_record_into<'out, T: CheckedRecord>( input: &[u8], restoration: &mut [u8], out: &'out mut MaybeUninit<T>, ) -> QslResult<&'out mut T> {
```

## qsl/src/rt.rs

`qsl/src/rt.rs:12`

```rust
pub struct StaticRegions<const CAP: usize, const N: usize> {
```

`qsl/src/rt.rs:25`

```rust
pub const fn new() -> Self {
```

`qsl/src/rt.rs:32`

```rust
pub fn try_acquire(&'static self) -> QslResult<RegionLease<CAP, N>> {
```

`qsl/src/rt.rs:58`

```rust
pub const fn new_zeroed() -> Self {
```

`qsl/src/rt.rs:66`

```rust
pub struct RegionLease<const CAP: usize, const N: usize> {
```

`qsl/src/rt.rs:72`

```rust
pub fn capacity(&self) -> usize {
```

`qsl/src/rt.rs:75`

```rust
pub fn as_slice(&self) -> &[u8] {
```

`qsl/src/rt.rs:84`

```rust
pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
```

`qsl/src/rt.rs:89`

```rust
pub unsafe fn set_len(&mut self, len: usize) {
```

## qsl/src/rust_macro.rs

`qsl/src/rust_macro.rs:200`

```rust
pub fn parse_rust_macros(sources: &[&str]) -> Result<Vec<QslSchema>, MacroError> {
```

`qsl/src/rust_macro.rs:312`

```rust
pub fn expand_with_rust( schemas: &[QslSchema], sources: &[&str], ) -> Result<Vec<QslSchema>, MacroError> {
```

`qsl/src/rust_macro.rs:354`

```rust
pub fn parse_qsl_with_rust( source: &str, constants: &HashMap<String, i128>, rust_sources: &[&str], ) -> Result<Vec<QslSchema>, MacroError> {
```

## qsl/src/schema_hash.rs

`qsl/src/schema_hash.rs:7`

```rust
pub const SCHEMA_HASH_VERSION: u16 = 1;
```

`qsl/src/schema_hash.rs:11`

```rust
pub enum SchemaHashAlgorithm {
```

`qsl/src/schema_hash.rs:19`

```rust
pub const fn bits(self) -> u32 {
```

`qsl/src/schema_hash.rs:25`

```rust
pub const fn name(self) -> &'static str {
```

`qsl/src/schema_hash.rs:31`

```rust
pub const fn rust_variant(self) -> &'static str {
```

`qsl/src/schema_hash.rs:56`

```rust
pub enum SchemaHashKind {
```

`qsl/src/schema_hash.rs:65`

```rust
pub struct SchemaHashes {
```

`qsl/src/schema_hash.rs:75`

```rust
pub fn strict_hex(self) -> String {
```

`qsl/src/schema_hash.rs:78`

```rust
pub fn light_hex(self) -> String {
```

`qsl/src/schema_hash.rs:90`

```rust
pub trait SchemaIdentity {
```

`qsl/src/schema_hash.rs:97`

```rust
pub fn hashes(&self, algorithm: SchemaHashAlgorithm) -> QslResult<SchemaHashes> {
```

`qsl/src/schema_hash.rs:112`

```rust
pub fn hash_descriptor(&self, kind: SchemaHashKind) -> QslResult<Vec<u8>> {
```

## qsl/src/schema_macro.rs

`qsl/src/schema_macro.rs:7`

```rust
pub struct MacroError(pub String);
```

`qsl/src/schema_macro.rs:15`

```rust
pub fn expand_macros(schemas: &[QslSchema]) -> Result<Vec<QslSchema>, MacroError> {
```

## qsl/src/serialize.rs

`qsl/src/serialize.rs:3`

```rust
pub trait Serializable: Sized {
```

`qsl/src/serialize.rs:10`

```rust
fn get_size() -> usize;
```

`qsl/src/serialize.rs:15`

```rust
fn serialized_size(&self) -> usize;
```

`qsl/src/serialize.rs:21`

```rust
unsafe fn serialize(&self) -> SerOut {
```

`qsl/src/serialize.rs:35`

```rust
unsafe fn serialize(&self) -> Vec<u8> {
```

`qsl/src/serialize.rs:48`

```rust
unsafe fn serialize_unsafe(&self, to: *mut u8);
```

`qsl/src/serialize.rs:54`

```rust
unsafe fn serialize_into_unsafe(&self, to: *mut u8) -> usize {
```

`qsl/src/serialize.rs:63`

```rust
unsafe fn serialize_to(&self, to: &mut SerOut) -> usize {
```

`qsl/src/serialize.rs:73`

```rust
unsafe fn serialize_to(&self, to: &mut [u8]) -> usize {
```

`qsl/src/serialize.rs:80`

```rust
unsafe fn deserialize(input: &[u8]) -> Self {
```

`qsl/src/serialize.rs:89`

```rust
unsafe fn deserialize_into(bytes: *const u8, len: usize, out: *mut Self) {
```

`qsl/src/serialize.rs:99`

```rust
unsafe fn deserialize_reuse(input: &[u8], output: &mut Self) {
```

`qsl/src/serialize.rs:107`

```rust
unsafe fn deserialize_prepared_into(bytes: *const u8, len: usize, output: &mut Self) {
```

`qsl/src/serialize.rs:115`

```rust
unsafe fn deserialize_unsafe(bytes: *const u8) -> Self;
```

`qsl/src/serialize.rs:120`

```rust
unsafe fn deserialize_from_raw_parts(bytes: *const u8, _len: usize) -> Self {
```

`qsl/src/serialize.rs:127`

```rust
pub trait BorrowSerializable<'a>: Sized {
```

`qsl/src/serialize.rs:133`

```rust
fn get_size() -> usize;
```

`qsl/src/serialize.rs:135`

```rust
fn serialized_size(&self) -> usize;
```

`qsl/src/serialize.rs:141`

```rust
unsafe fn serialize(&self) -> SerOut {
```

`qsl/src/serialize.rs:153`

```rust
unsafe fn serialize(&self) -> Vec<u8> {
```

`qsl/src/serialize.rs:165`

```rust
unsafe fn serialize_unsafe(&self, to: *mut u8);
```

`qsl/src/serialize.rs:171`

```rust
unsafe fn serialize_into_unsafe(&self, to: *mut u8) -> usize {
```

`qsl/src/serialize.rs:179`

```rust
unsafe fn serialize_to(&self, to: &mut SerOut) -> usize {
```

`qsl/src/serialize.rs:188`

```rust
unsafe fn serialize_to(&self, to: &mut [u8]) -> usize {
```

`qsl/src/serialize.rs:194`

```rust
unsafe fn deserialize_borrowed(input: &'a [u8]) -> Self {
```

`qsl/src/serialize.rs:203`

```rust
unsafe fn deserialize_borrowed_into(bytes: *const u8, len: usize, out: *mut Self) {
```

`qsl/src/serialize.rs:210`

```rust
unsafe fn deserialize_borrowed_unsafe(bytes: *const u8) -> Self;
```

`qsl/src/serialize.rs:215`

```rust
unsafe fn deserialize_borrowed_from_raw_parts(bytes: *const u8, _len: usize) -> Self {
```

`qsl/src/serialize.rs:220`

```rust
pub struct SerOut {
```

`qsl/src/serialize.rs:228`

```rust
pub fn new(len: usize) -> Self {
```

`qsl/src/serialize.rs:247`

```rust
pub fn with_capacity_aligned(capacity: usize, alignment: usize) -> Self {
```

`qsl/src/serialize.rs:264`

```rust
pub fn capacity(&self) -> usize {
```

`qsl/src/serialize.rs:268`

```rust
pub fn alignment(&self) -> usize {
```

`qsl/src/serialize.rs:272`

```rust
pub fn clear(&mut self) {
```

`qsl/src/serialize.rs:279`

```rust
pub unsafe fn set_len(&mut self, len: usize) {
```

`qsl/src/serialize.rs:287`

```rust
pub unsafe fn from_raw(ptr: *mut u8, layout: std::alloc::Layout, len: usize) -> Self {
```

`qsl/src/serialize.rs:297`

```rust
pub fn as_mut_ptr(&mut self) -> *mut u8 {
```

`qsl/src/serialize.rs:342`

```rust
pub fn len(&self) -> usize {
```

`qsl/src/serialize.rs:346`

```rust
pub fn is_empty(&self) -> bool {
```

`qsl/src/serialize.rs:351`

```rust
pub fn copy_from_slice(&mut self, bytes: &[u8]) -> crate::QslResult<()> {
```

`qsl/src/serialize.rs:365`

```rust
pub unsafe fn copy_from_raw_parts(&mut self, source: *const u8, len: usize) {
```

`qsl/src/serialize.rs:370`

```rust
pub fn spare_capacity_mut(&mut self) -> &mut [std::mem::MaybeUninit<u8>] {
```

`qsl/src/serialize.rs:380`

```rust
pub fn into_utf8_guard(self) -> Result<crate::Utf8CopyGuard, (std::str::Utf8Error, Self)> {
```

`qsl/src/serialize.rs:386`

```rust
pub trait Sealed {
```

`qsl/src/serialize.rs:393`

```rust
pub trait DeserializeDestination<T: Serializable>: destination_private::Sealed {
```

`qsl/src/serialize.rs:398`

```rust
unsafe fn decode_into(self, bytes: *const u8, len: usize);
```

## qsl/src/socket.rs

`qsl/src/socket.rs:10`

```rust
pub struct RawSocket {
```

`qsl/src/socket.rs:15`

```rust
pub fn new(stream: TcpStream, max_frame: usize) -> io::Result<Self> {
```

`qsl/src/socket.rs:34`

```rust
pub fn receive<'a>(&mut self, storage: &'a mut [u8]) -> io::Result<&'a [u8]> {
```

`qsl/src/socket.rs:74`

```rust
pub unsafe fn send_record<T: Serializable>( &mut self, value: &T, storage: &mut [u8], ) -> io::Result<usize> {
```

`qsl/src/socket.rs:85`

```rust
pub unsafe fn send_borrowed_record<'a, T: BorrowSerializable<'a>>( &mut self, value: &T, storage: &mut [u8], ) -> io::Result<usize> {
```

`qsl/src/socket.rs:94`

```rust
pub fn stream(&self) -> &TcpStream {
```

## qsl/src/static_index.rs

`qsl/src/static_index.rs:6`

```rust
pub const ANALYSIS_MAX_STATES: usize = 4096;
```

`qsl/src/static_index.rs:7`

```rust
pub const ANALYSIS_MAX_BYTES: usize = 65535;
```

`qsl/src/static_index.rs:10`

```rust
pub struct StateCode {
```

`qsl/src/static_index.rs:18`

```rust
pub struct OffsetRow {
```

`qsl/src/static_index.rs:24`

```rust
pub struct PredictPlan {
```

`qsl/src/static_index.rs:34`

```rust
pub fn analyze(schema: &QslSchema) -> QslResult<Self> {
```

`qsl/src/static_index.rs:149`

```rust
pub fn packed_index(&self) -> QslResult<Vec<u32>> {
```

`qsl/src/static_index.rs:153`

```rust
pub fn pack_lengths(&self, lengths: &[Option<usize>]) -> QslResult<u32> {
```

`qsl/src/static_index.rs:169`

```rust
pub fn unpack_lengths(&self, d: u32) -> QslResult<Vec<Option<usize>>> {
```

`qsl/src/static_index.rs:201`

```rust
pub fn quantize_index(index: &[u32], quantum: usize) -> QslResult<Vec<u32>> {
```

`qsl/src/static_index.rs:229`

```rust
pub fn lookup_index(table: &[u32], quantum: usize, index: usize) -> Option<u32> {
```

`qsl/src/static_index.rs:243`

```rust
pub fn structural_counts(domains: &[Vec<usize>], max_len: usize) -> QslResult<Vec<u64>> {
```

## qsl/src/static_optional.rs

`qsl/src/static_optional.rs:4`

```rust
pub enum StaticPresence {
```

`qsl/src/static_optional.rs:11`

```rust
pub struct StaticOptionalPlan {
```

`qsl/src/static_optional.rs:16`

```rust
pub fn analyze(schema: &QslSchema) -> QslResult<Self> {
```

`qsl/src/static_optional.rs:75`

```rust
pub fn header_bytes(&self) -> usize {
```

## qsl/src/strings.rs

`qsl/src/strings.rs:12`

```rust
pub const STRING_COMPRESSION_MIN_BYTES: usize = 32;
```

`qsl/src/strings.rs:15`

```rust
pub const STRING_ENCODER_REVISION: u8 = 2;
```

`qsl/src/strings.rs:20`

```rust
pub trait Sealed {
```

`qsl/src/strings.rs:28`

```rust
pub trait StringHeader: header_sealed::Sealed {
```

`qsl/src/strings.rs:33`

```rust
unsafe fn load(input: *const u8) -> u32;
```

`qsl/src/strings.rs:36`

```rust
unsafe fn store(output: *mut u8, value: u32);
```

`qsl/src/strings.rs:64`

```rust
pub struct InlineString<const MAX: usize> {
```

`qsl/src/strings.rs:71`

```rust
pub const MAX_BYTES: usize = MAX;
```

`qsl/src/strings.rs:73`

```rust
pub const HEADER_BYTES: usize = if MAX <= 127 {
```

`qsl/src/strings.rs:81`

```rust
pub const OFFSET_BYTES: usize = if MAX <= 255 {
```

`qsl/src/strings.rs:86`

```rust
pub const fn new() -> Self {
```

`qsl/src/strings.rs:98`

```rust
pub unsafe fn from_str_unchecked(value: &str) -> Self {
```

`qsl/src/strings.rs:112`

```rust
pub const fn spaces<const LEN: usize>() -> Self {
```

`qsl/src/strings.rs:126`

```rust
pub fn len(&self) -> usize {
```

`qsl/src/strings.rs:132`

```rust
pub fn is_empty(&self) -> bool {
```

`qsl/src/strings.rs:138`

```rust
pub fn as_str(&self) -> &str {
```

`qsl/src/strings.rs:150`

```rust
pub fn wire_capacity(&self) -> usize {
```

`qsl/src/strings.rs:159`

```rust
pub unsafe fn encode_to(&self, output: *mut u8) -> usize {
```

`qsl/src/strings.rs:167`

```rust
pub unsafe fn encode_to_with<H: StringHeader>(&self, output: *mut u8) -> usize {
```

`qsl/src/strings.rs:175`

```rust
pub unsafe fn encode_raw_to(&self, output: *mut u8) -> usize {
```

`qsl/src/strings.rs:190`

```rust
pub unsafe fn decode_into(input: *const u8, output: *mut Self) -> usize {
```

`qsl/src/strings.rs:200`

```rust
pub unsafe fn decode_into_with<H: StringHeader>(input: *const u8, output: *mut Self) -> usize {
```

`qsl/src/strings.rs:211`

```rust
pub unsafe fn decode_raw_into_with<H: StringHeader>( input: *const u8, output: *mut Self, ) -> usize {
```

`qsl/src/strings.rs:232`

```rust
pub unsafe fn decode_from_with<H: StringHeader>(input: *const u8) -> (Self, usize) {
```

`qsl/src/strings.rs:242`

```rust
pub unsafe fn decode_raw_from_with<H: StringHeader>(input: *const u8) -> (Self, usize) {
```

`qsl/src/strings.rs:252`

```rust
pub unsafe fn decode_raw_into(input: *const u8, output: *mut Self) -> usize {
```

`qsl/src/strings.rs:269`

```rust
pub unsafe fn decode_from(input: *const u8) -> (Self, usize) {
```

`qsl/src/strings.rs:421`

```rust
pub unsafe fn encode_bounded<const MAX: usize>( input: *const u8, len: usize, output: *mut u8, ) -> usize {
```

`qsl/src/strings.rs:439`

```rust
pub unsafe fn encode_bounded_with<const MAX: usize, H: StringHeader>( input: *const u8, len: usize, output: *mut u8, ) -> usize {
```

`qsl/src/strings.rs:761`

```rust
pub unsafe fn decode_bounded<const MAX: usize>( input: *const u8, output: *mut u8, ) -> (usize, usize) {
```

`qsl/src/strings.rs:778`

```rust
pub unsafe fn decode_bounded_with<const MAX: usize, H: StringHeader>( input: *const u8, output: *mut u8, ) -> (usize, usize) {
```

`qsl/src/strings.rs:866`

```rust
pub const FIXED_UTF8_CODEC_VERSION: u8 = 2;
```

`qsl/src/strings.rs:874`

```rust
pub unsafe fn encode_fixed_utf8<const MAX: usize>(input: *const u8, output: *mut u8) -> usize {
```

`qsl/src/strings.rs:896`

```rust
pub unsafe fn decode_fixed_utf8<const MAX: usize>(input: *const u8, output: *mut u8) -> usize {
```

`qsl/src/strings.rs:918`

```rust
pub struct Utf8Guard<'a> {
```

`qsl/src/strings.rs:923`

```rust
pub const fn from_str(value: &'a str) -> Self {
```

`qsl/src/strings.rs:929`

```rust
pub fn try_from_bytes(bytes: &'a [u8]) -> Result<Self, str::Utf8Error> {
```

`qsl/src/strings.rs:940`

```rust
pub const unsafe fn from_bytes_unchecked(bytes: &'a [u8]) -> Self {
```

`qsl/src/strings.rs:944`

```rust
pub const fn as_bytes(self) -> &'a [u8] {
```

`qsl/src/strings.rs:948`

```rust
pub const fn as_str(self) -> &'a str {
```

`qsl/src/strings.rs:1326`

```rust
pub type Utf8View<'a> = Utf8Guard<'a>;
```

`qsl/src/strings.rs:1333`

```rust
pub struct Utf8CopyGuard {
```

`qsl/src/strings.rs:1337`

```rust
pub fn with_capacity(capacity: usize) -> Self {
```

`qsl/src/strings.rs:1342`

```rust
pub fn copy_of(value: &str) -> Self {
```

`qsl/src/strings.rs:1349`

```rust
pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, str::Utf8Error> {
```

`qsl/src/strings.rs:1356`

```rust
pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> Self {
```

`qsl/src/strings.rs:1362`

```rust
pub fn capacity(&self) -> usize {
```

`qsl/src/strings.rs:1366`

```rust
pub fn len(&self) -> usize {
```

`qsl/src/strings.rs:1370`

```rust
pub fn is_empty(&self) -> bool {
```

`qsl/src/strings.rs:1374`

```rust
pub fn as_bytes(&self) -> &[u8] {
```

`qsl/src/strings.rs:1378`

```rust
pub fn as_str(&self) -> &str {
```

`qsl/src/strings.rs:1381`

```rust
pub fn clear(&mut self) {
```

`qsl/src/strings.rs:1386`

```rust
pub fn copy_from_str(&mut self, value: &str) -> crate::QslResult<()> {
```

`qsl/src/strings.rs:1390`

```rust
pub fn try_copy_from_bytes(&mut self, bytes: &[u8]) -> crate::QslResult<()> {
```

`qsl/src/strings.rs:1404`

```rust
pub unsafe fn copy_from_raw_parts_unchecked(&mut self, source: *const u8, len: usize) {
```

`qsl/src/strings.rs:1412`

```rust
pub unsafe fn decode_bounded_into<const MAX: usize>(&mut self, input: *const u8) -> usize {
```

`qsl/src/strings.rs:1418`

```rust
pub fn try_from_ser_out( storage: crate::SerOut, ) -> Result<Self, (str::Utf8Error, crate::SerOut)> {
```

`qsl/src/strings.rs:1428`

```rust
pub unsafe fn from_ser_out_unchecked(storage: crate::SerOut) -> Self {
```

`qsl/src/strings.rs:1432`

```rust
pub fn into_ser_out(self) -> crate::SerOut {
```

## qsl/src/types.rs

`qsl/src/types.rs:4`

```rust
pub const FLAG_COMPACT: usize = 0b0000_0001;
```

`qsl/src/types.rs:5`

```rust
pub const FLAG_SAFETY: usize = 0b0000_0010;
```

`qsl/src/types.rs:6`

```rust
pub const FLAG_ALIGNMENT: usize = 0b0000_0100;
```

`qsl/src/types.rs:7`

```rust
pub const FLAG_OPTIONAL_BITMAP: usize = 0b0001_0000;
```

`qsl/src/types.rs:8`

```rust
pub const FLAG_OPT_TABLE: usize = 0b0010_0000;
```

`qsl/src/types.rs:9`

```rust
pub const FLAG_PREDICT_SEQ: usize = 0b0100_0000;
```

`qsl/src/types.rs:11`

```rust
pub const FLAG_FIELD_BORROW: usize = 0b1000_0000;
```

`qsl/src/types.rs:12`

```rust
pub const FLAG_LIFETIME: usize = 0b0000_1000;
```

`qsl/src/types.rs:13`

```rust
pub const FLAG_STATE_BITS: usize = 1 << 8;
```

`qsl/src/types.rs:14`

```rust
pub const FLAG_INLINE: usize = 1 << 9;
```

`qsl/src/types.rs:15`

```rust
pub const FLAG_INLINE_ALWAYS: usize = 1 << 10;
```

`qsl/src/types.rs:16`

```rust
pub const FLAG_INLINE_NEVER: usize = 1 << 11;
```

`qsl/src/types.rs:17`

```rust
pub const STATE_BITS_QUANTUM_SHIFT: usize = 12;
```

`qsl/src/types.rs:18`

```rust
pub const STATE_BITS_QUANTUM_MASK: usize = 7 << STATE_BITS_QUANTUM_SHIFT;
```

`qsl/src/types.rs:20`

```rust
pub const FLAG_MACRO: usize = 1 << 15;
```

`qsl/src/types.rs:22`

```rust
pub const FLAG_OPT_TABLE_LEGACY: usize = 1 << 16;
```

`qsl/src/types.rs:23`

```rust
pub const FLAG_PAYLOAD_QUANT: usize = 1 << 17;
```

`qsl/src/types.rs:25`

```rust
pub const FLAG_QPS: usize = FLAG_PAYLOAD_QUANT;
```

`qsl/src/types.rs:26`

```rust
pub const PAYLOAD_QUANT_SHIFT: usize = 18;
```

`qsl/src/types.rs:27`

```rust
pub const PAYLOAD_QUANT_MASK: usize = 3 << PAYLOAD_QUANT_SHIFT;
```

`qsl/src/types.rs:34`

```rust
pub trait QslCustom: Sized {
```

`qsl/src/types.rs:43`

```rust
fn qsl_validate(_bytes: &[u8], _compact: bool) -> QslResult<()> {
```

`qsl/src/types.rs:49`

```rust
fn qsl_validate_values(&self) -> QslResult<()> {
```

`qsl/src/types.rs:56`

```rust
unsafe fn qsl_write_unchecked(&self, to: *mut u8);
```

`qsl/src/types.rs:61`

```rust
unsafe fn qsl_read_unchecked(from: *const u8) -> Self;
```

`qsl/src/types.rs:65`

```rust
fn qsl_size(&self) -> usize {
```

`qsl/src/types.rs:71`

```rust
fn qsl_comp_size(&self) -> usize {
```

`qsl/src/types.rs:78`

```rust
unsafe fn qsl_comp_write_unchecked(&self, to: *mut u8) {
```

`qsl/src/types.rs:85`

```rust
unsafe fn qsl_comp_read_unchecked(from: *const u8) -> Self {
```

`qsl/src/types.rs:91`

```rust
pub enum QslType {
```

`qsl/src/types.rs:160`

```rust
pub fn rank_bits(min: i128, max: i128) -> QslResult<usize> {
```

`qsl/src/types.rs:170`

```rust
pub fn rank_width(&self, fallback: usize) -> QslResult<usize> {
```

`qsl/src/types.rs:180`

```rust
pub fn rank_base(&self) -> i128 {
```

`qsl/src/types.rs:192`

```rust
pub fn number16(signed: bool, min: usize) -> Self {
```

`qsl/src/types.rs:199`

```rust
pub fn number32(signed: bool, min: usize) -> Self {
```

`qsl/src/types.rs:206`

```rust
pub fn number64(signed: bool, min: usize) -> Self {
```

`qsl/src/types.rs:213`

```rust
pub fn primitive(&self) -> QslResult<Option<&'static str>> {
```

`qsl/src/types.rs:241`

```rust
pub fn rust_type(&self, borrowed: bool) -> QslResult<String> {
```

`qsl/src/types.rs:273`

```rust
pub fn fixed_wire_size(&self) -> QslResult<Option<usize>> {
```

`qsl/src/types.rs:312`

```rust
pub fn default_compact_bits(&self) -> QslResult<usize> {
```

`qsl/src/types.rs:330`

```rust
pub fn validate(&self, borrowed: bool) -> QslResult<()> {
```

`qsl/src/types.rs:385`

```rust
pub struct QslField {
```

`qsl/src/types.rs:399`

```rust
pub struct QslSchema {
```

`qsl/src/types.rs:407`

```rust
pub fn compact(&self) -> bool {
```

`qsl/src/types.rs:410`

```rust
pub fn bitmap(&self) -> bool {
```

`qsl/src/types.rs:413`

```rust
pub fn opt_table(&self) -> bool {
```

`qsl/src/types.rs:417`

```rust
pub fn qps(&self) -> bool {
```

`qsl/src/types.rs:420`

```rust
pub fn quantum(&self) -> usize {
```

`qsl/src/types.rs:423`

```rust
pub fn quant_requested(&self) -> bool {
```

`qsl/src/types.rs:426`

```rust
pub fn payload_quantum(&self) -> usize {
```

`qsl/src/types.rs:429`

```rust
pub fn legacy_opt_table(&self) -> bool {
```

`qsl/src/types.rs:432`

```rust
pub fn predict_seq(&self) -> bool {
```

`qsl/src/types.rs:437`

```rust
pub fn state_bits(&self) -> bool {
```

`qsl/src/types.rs:440`

```rust
pub fn state_bits_quantum(&self) -> usize {
```

`qsl/src/types.rs:443`

```rust
pub fn predict_table(&self) -> bool {
```

`qsl/src/types.rs:447`

```rust
pub fn predicted_opt_table(&self) -> bool {
```

`qsl/src/types.rs:450`

```rust
pub fn sequential(&self) -> bool {
```

`qsl/src/types.rs:453`

```rust
pub fn lifetime(&self) -> bool {
```

`qsl/src/types.rs:456`

```rust
pub fn borrow_strings(&self) -> bool {
```

`qsl/src/types.rs:459`

```rust
pub fn optional_count(&self) -> usize {
```

`qsl/src/types.rs:463`

```rust
pub fn tag_bits(&self) -> usize {
```

`qsl/src/types.rs:471`

```rust
pub fn count_bits(&self) -> usize {
```

`qsl/src/types.rs:480`

```rust
pub fn bitmap_words(&self) -> usize {
```

`qsl/src/types.rs:483`

```rust
pub fn validate(&self) -> QslResult<()> {
```

## qsl/src/validation.rs

`qsl/src/validation.rs:5`

```rust
pub const DEFAULT_MAX_DATA_SIZE: usize = 65535;
```

`qsl/src/validation.rs:7`

```rust
pub fn check_size(size: usize, maximum: usize) -> QslResult<()> {
```

`qsl/src/validation.rs:16`

```rust
pub struct ValueBudget {
```

`qsl/src/validation.rs:20`

```rust
pub fn new(maximum: usize) -> Self {
```

`qsl/src/validation.rs:23`

```rust
pub fn charge(&mut self, bytes: usize) -> QslResult<()> {
```

`qsl/src/validation.rs:32`

```rust
pub struct ValidationCursor<'a> {
```

`qsl/src/validation.rs:38`

```rust
pub fn new(input: &'a [u8]) -> Self {
```

`qsl/src/validation.rs:46`

```rust
pub fn with_limit(input: &'a [u8], maximum: usize) -> QslResult<Self> {
```

`qsl/src/validation.rs:54`

```rust
pub fn charge(&mut self, bytes: usize) -> QslResult<()> {
```

`qsl/src/validation.rs:57`

```rust
pub fn payload(&mut self, bytes: usize) -> QslResult<&'a [u8]> {
```

`qsl/src/validation.rs:61`

```rust
pub fn bits(&mut self, width: usize) -> QslResult<u64> {
```

`qsl/src/validation.rs:86`

```rust
pub fn align(&mut self) -> QslResult<()> {
```

`qsl/src/validation.rs:93`

```rust
pub fn position(&self) -> usize {
```

`qsl/src/validation.rs:96`

```rust
pub fn bytes(&mut self, n: usize) -> QslResult<&'a [u8]> {
```

`qsl/src/validation.rs:104`

```rust
pub fn uint(&mut self, n: usize) -> QslResult<u64> {
```

`qsl/src/validation.rs:112`

```rust
pub fn utf8(&mut self, n: usize, min: usize, max: usize) -> QslResult<()> {
```

`qsl/src/validation.rs:119`

```rust
pub fn finish(mut self) -> QslResult<()> {
```

`qsl/src/validation.rs:128`

```rust
pub fn compressed<const MAX: usize>( &mut self, min: usize, raw_only: bool, fixed: bool, ) -> QslResult<()> {
```
