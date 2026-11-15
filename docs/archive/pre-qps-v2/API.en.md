# QSL public API — source-extracted index
> Signatures are extracted from the uploaded QSL.rar source. This is a source index, not a build-verified compatibility promise for every Cargo feature combination. Documentation comments are preserved from Rust sources. Unsafe APIs require the documented caller invariants.

## `compat.rs`

### `pub enum ParserQslType` · L10

```rust
pub enum ParserQslType
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub struct ParserQslField` · L25

```rust
pub struct ParserQslField
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub struct ParserQslSchema` · L34

```rust
pub struct ParserQslSchema
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn new` · L42

```rust
pub fn new(flags: usize) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn optional_count` · L48

```rust
pub fn optional_count(&self) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn validate` · L51

```rust
pub fn validate(&self) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.


## `deq.rs`

### `pub struct DeqConstant` · L14

```rust
pub struct DeqConstant
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub struct DeqConfig` · L20

```rust
pub struct DeqConfig
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn integer_values` · L27

```rust
pub fn integer_values(&self) -> HashMap<String, i128>
```

QSL currently interprets numeric constant values at build time, while the DEQ AST retains the source type for validation and diagnostics.

### `pub fn parse_deq` · L35

```rust
pub fn parse_deq(source: &str) -> Result<DeqConfig, ParseError>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn build_qsl_files` · L89

```rust
pub fn build_qsl_files( qsl_files: &[&str], deq_files: &[&str], out_dir: impl AsRef<Path>, ) -> Result<PathBuf, Box<dyn std::error::Error>>
```

Explicit build inputs belong in build.rs; DEQ itself only provides values. Lists are traversed in caller order, preserving deterministic schema order.


## `interpreter.rs`

### `pub trait TypeInterpreter` · L107

```rust
pub trait TypeInterpreter
```

The code generator does all stable arithmetic before emitting Rust source. The same type-specific routines are used by the schema writer and reader.

### `pub struct QslInterpreter` · L1748

```rust
pub struct QslInterpreter
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn new` · L1775

```rust
pub fn new() -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn generate` · L1779

```rust
pub fn generate(&mut self, schemas: &[QslSchema]) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn generate_parsed` · L1797

```rust
pub fn generate_parsed(&mut self, schemas: &[crate::compat::ParserQslSchema]) -> QslResult<()>
```

Parser-compatible front end. QSL parser AST stays untouched; only the internal wire-generation descriptors are normalized here.

### `pub fn generate_field` · L1805

```rust
pub fn generate_field(&mut self, field: &QslField, borrowed: bool) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn generate_default` · L2028

```rust
pub fn generate_default(&mut self, schema: &QslSchema) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn generate_lifetime` · L2039

```rust
pub fn generate_lifetime(&mut self, schema: &QslSchema) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn push` · L2053

```rust
pub fn push(&self) -> std::io::Result<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.


## `io.rs`

### `pub type QslResult` · L9

```rust
pub type QslResult<T> = Result<T, QslError>;
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub enum QslError` · L12

```rust
pub enum QslError
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub struct QslRawWriter` · L33

```rust
pub struct QslRawWriter
```

Minimal raw-pointer writer used by trusted, compile-time-validated schemas. No bounds checks, dynamic allocation, loops, or error values exist here. The caller is responsible for the complete writable allocation.

### `pub unsafe fn new` · L41

```rust
pub unsafe fn new(ptr: *mut u8) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn from` · L48

```rust
pub fn from(out: &mut [u8]) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_u8` · L58

```rust
pub unsafe fn write_u8(&mut self, value: u8)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_u16` · L66

```rust
pub unsafe fn write_u16(&mut self, value: u16)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_u32` · L74

```rust
pub unsafe fn write_u32(&mut self, value: u32)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_u64` · L82

```rust
pub unsafe fn write_u64(&mut self, value: u64)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write` · L90

```rust
pub unsafe fn write(&mut self, source: *const u8, len: usize)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub struct QslRawReader` · L98

```rust
pub struct QslRawReader
```

Minimal raw-pointer reader. Never checks the input length or its contents. Any use with a truncated or invalid allocation is the caller's fault.

### `pub unsafe fn new` · L106

```rust
pub unsafe fn new(ptr: *const u8) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn from` · L112

```rust
pub fn from(input: &[u8]) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn read_u8` · L121

```rust
pub unsafe fn read_u8(&mut self) -> u8
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn read_u16` · L130

```rust
pub unsafe fn read_u16(&mut self) -> u16
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn read_u32` · L139

```rust
pub unsafe fn read_u32(&mut self) -> u32
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn read_u64` · L148

```rust
pub unsafe fn read_u64(&mut self) -> u64
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn take` · L156

```rust
pub unsafe fn take(&mut self, len: usize) -> *const u8
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub struct QslFastBitsReader` · L166

```rust
pub struct QslFastBitsReader
```

Unchecked LSB-first bit reader for trusted records. It deliberately has no length, Result, UTF-8, or padding checks. A u128 register moves exactly 0..=8 bytes per operation without bit-by-bit loops or branches.

### `pub fn as_ptr` · L175

```rust
pub fn as_ptr(&self) -> *const u8
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn read_raw_inline_string_into` · L183

```rust
pub unsafe fn read_raw_inline_string_into<const MAX: usize>( &mut self, output: *mut crate::InlineString<MAX>, )
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn read_inline_string_into` · L195

```rust
pub unsafe fn read_inline_string_into<const MAX: usize>( &mut self, output: *mut crate::InlineString<MAX>, )
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn read_inline_string` · L207

```rust
pub unsafe fn read_inline_string<const MAX: usize>(&mut self) -> crate::InlineString<MAX>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn new` · L218

```rust
pub unsafe fn new(ptr: *const u8) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn from` · L229

```rust
pub fn from(input: &[u8]) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn read_bits` · L243

```rust
pub unsafe fn read_bits(&mut self, width: u32) -> u64
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn align` · L262

```rust
pub unsafe fn align(&mut self)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn read_exact` · L271

```rust
pub unsafe fn read_exact(&mut self, len: usize) -> *const u8
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn read_u8` · L283

```rust
pub unsafe fn read_u8(&mut self) -> u8
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn read_u16` · L294

```rust
pub unsafe fn read_u16(&mut self) -> u16
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn read_u32` · L305

```rust
pub unsafe fn read_u32(&mut self) -> u32
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn read_u64` · L316

```rust
pub unsafe fn read_u64(&mut self) -> u64
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn read_i8` · L326

```rust
pub unsafe fn read_i8(&mut self) -> i8
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn read_i16` · L333

```rust
pub unsafe fn read_i16(&mut self) -> i16
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn read_i32` · L340

```rust
pub unsafe fn read_i32(&mut self) -> i32
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn read_i64` · L347

```rust
pub unsafe fn read_i64(&mut self) -> i64
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub struct QslUncheckedWriter` · L359

```rust
pub struct QslUncheckedWriter
```

Unchecked hot-path writer used after static schema analysis and exact buffer allocation. Values are trusted; no runtime rank preflight is performed. No capacity checks or value checks occur during the write pass.  The caller must provide a writable allocation large enough for the entire encoded record, and all values must satisfy the declared schem

### `pub fn as_mut_ptr` · L368

```rust
pub fn as_mut_ptr(&self) -> *mut u8
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_inline_string` · L375

```rust
pub unsafe fn write_inline_string<const MAX: usize>( &mut self, value: &crate::InlineString<MAX>, )
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn from_raw` · L386

```rust
pub unsafe fn from_raw(ptr: *mut u8) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn from` · L397

```rust
pub fn from(out: &mut [u8]) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_bytes_unchecked` · L409

```rust
pub unsafe fn write_bytes_unchecked(&mut self, bytes: &[u8])
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_ptr_unchecked` · L419

```rust
pub unsafe fn write_ptr_unchecked(&mut self, source: *const u8, len: usize)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_zeros_unchecked` · L429

```rust
pub unsafe fn write_zeros_unchecked(&mut self, count: usize)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_bits_unchecked` · L439

```rust
pub unsafe fn write_bits_unchecked(&mut self, value: u64, width: u32)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn flush_unchecked` · L456

```rust
pub unsafe fn flush_unchecked(&mut self)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn claim_bytes` · L471

```rust
pub unsafe fn claim_bytes(&mut self, n: usize) -> *mut u8
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_bytes` · L482

```rust
pub unsafe fn write_bytes(&mut self, bytes: &[u8])
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_zeros` · L490

```rust
pub unsafe fn write_zeros(&mut self, n: usize)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_bits` · L498

```rust
pub unsafe fn write_bits(&mut self, v: u64, width: u32)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn flush` · L507

```rust
pub unsafe fn flush(&mut self)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_u8` · L514

```rust
pub unsafe fn write_u8(&mut self, value: u8)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_u16` · L524

```rust
pub unsafe fn write_u16(&mut self, value: u16)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_u32` · L534

```rust
pub unsafe fn write_u32(&mut self, value: u32)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_u64` · L544

```rust
pub unsafe fn write_u64(&mut self, value: u64)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_i8` · L553

```rust
pub unsafe fn write_i8(&mut self, v: i8)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_i16` · L560

```rust
pub unsafe fn write_i16(&mut self, v: i16)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_i32` · L567

```rust
pub unsafe fn write_i32(&mut self, v: i32)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn write_i64` · L574

```rust
pub unsafe fn write_i64(&mut self, v: i64)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub type QslWriter = QslRawWriter;` · L581

```rust
pub type QslWriter = QslRawWriter;
```

Primary QSL I/O names intentionally resolve to the minimal unchecked cursors. The `QslChecked*` variants are for diagnostics and input validation.

### `pub type QslReader = QslRawReader;` · L582

```rust
pub type QslReader = QslRawReader;
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub type QslBitsWriter = QslUncheckedWriter;` · L583

```rust
pub type QslBitsWriter = QslUncheckedWriter;
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub type QslBitsReader = QslFastBitsReader;` · L584

```rust
pub type QslBitsReader = QslFastBitsReader;
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub struct QslSizer` · L588

```rust
pub struct QslSizer
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn bits` · L595

```rust
pub fn bits(&mut self, n: usize) -> QslResult<()>
```

Add `n` bits with checked overflow arithmetic, folding completed bytes. This utility is for code generation, not serialized object I/O.

### `pub fn align` · L605

```rust
pub fn align(&mut self) -> QslResult<()>
```

Round the pending bits up to the next byte boundary, rejecting overflow.

### `pub fn bytes` · L613

```rust
pub fn bytes(&mut self, n: usize) -> QslResult<()>
```

Append `n` whole bytes after byte alignment, rejecting overflow.

### `pub fn finish` · L619

```rust
pub fn finish(mut self) -> QslResult<usize>
```

Return the final whole-byte size, including the last partial byte.


## `io_checked.rs`

### `pub struct QslCheckedWriter` · L9

```rust
pub struct QslCheckedWriter<'a>
```

All writes use raw pointer copies. Safe constructors still bound-check them.

### `pub fn from` · L17

```rust
pub fn from(out: &'a mut [u8]) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn from_raw_parts` · L22

```rust
pub unsafe fn from_raw_parts(ptr: *mut u8, len: usize) -> Self
```

`ptr` must be writable for `len` bytes, exclusively, throughout `'a`.

### `pub fn from_raw` · L31

```rust
pub fn from_raw(out: &'a mut crate::serialize::SerOut) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn position` · L34

```rust
pub fn position(&self) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write` · L37

```rust
pub fn write(&mut self, bytes: &[u8]) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_zeros` · L53

```rust
pub fn write_zeros(&mut self, len: usize) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_u8` · L66

```rust
pub fn write_u8(&mut self, v: u8) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_u16` · L69

```rust
pub fn write_u16(&mut self, v: u16) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_u32` · L72

```rust
pub fn write_u32(&mut self, v: u32) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_u64` · L75

```rust
pub fn write_u64(&mut self, v: u64) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_i8` · L78

```rust
pub fn write_i8(&mut self, v: i8) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_i16` · L81

```rust
pub fn write_i16(&mut self, v: i16) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_i32` · L84

```rust
pub fn write_i32(&mut self, v: i32) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_i64` · L87

```rust
pub fn write_i64(&mut self, v: i64) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub struct QslCheckedReader` · L92

```rust
pub struct QslCheckedReader<'a>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn new` · L99

```rust
pub fn new(bytes: &'a [u8]) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn from_raw_parts` · L104

```rust
pub unsafe fn from_raw_parts(ptr: *const u8, len: usize) -> Self
```

`ptr` must be readable for `len` bytes throughout `'a`.

### `pub fn position` · L112

```rust
pub fn position(&self) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_bytes` · L115

```rust
pub fn read_bytes(&mut self, n: usize) -> QslResult<&'a [u8]>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_u8` · L124

```rust
pub fn read_u8(&mut self) -> QslResult<u8>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_u16` · L127

```rust
pub fn read_u16(&mut self) -> QslResult<u16>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_u32` · L130

```rust
pub fn read_u32(&mut self) -> QslResult<u32>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_u64` · L133

```rust
pub fn read_u64(&mut self) -> QslResult<u64>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_i8` · L136

```rust
pub fn read_i8(&mut self) -> QslResult<i8>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_i16` · L139

```rust
pub fn read_i16(&mut self) -> QslResult<i16>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_i32` · L142

```rust
pub fn read_i32(&mut self) -> QslResult<i32>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_i64` · L145

```rust
pub fn read_i64(&mut self) -> QslResult<i64>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub struct QslCheckedBitsWriter` · L152

```rust
pub struct QslCheckedBitsWriter<'a>
```

LSB-first bit stream. A value always occupies exactly `width` bits, independently of how many leading zeroes it contains.

### `pub fn from` · L161

```rust
pub fn from(out: &'a mut [u8]) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn from_raw_parts` · L166

```rust
pub unsafe fn from_raw_parts(ptr: *mut u8, len: usize) -> Self
```

`ptr` must be exclusively writable for `len` bytes throughout `'a`.

### `pub fn from_raw` · L177

```rust
pub fn from_raw(out: &'a mut crate::serialize::SerOut) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn position` · L180

```rust
pub fn position(&self) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_bits` · L193

```rust
pub fn write_bits(&mut self, mut value: u64, mut width: u32) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn flush` · L215

```rust
pub fn flush(&mut self) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_bytes` · L223

```rust
pub fn write_bytes(&mut self, bytes: &[u8]) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_zeros` · L240

```rust
pub fn write_zeros(&mut self, n: usize) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_u8` · L254

```rust
pub fn write_u8(&mut self, v: u8) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_u16` · L257

```rust
pub fn write_u16(&mut self, v: u16) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_u32` · L260

```rust
pub fn write_u32(&mut self, v: u32) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_u64` · L263

```rust
pub fn write_u64(&mut self, v: u64) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_i8` · L266

```rust
pub fn write_i8(&mut self, v: i8) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_i16` · L269

```rust
pub fn write_i16(&mut self, v: i16) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_i32` · L272

```rust
pub fn write_i32(&mut self, v: i32) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn write_i64` · L275

```rust
pub fn write_i64(&mut self, v: i64) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub struct QslCheckedBitsReader` · L280

```rust
pub struct QslCheckedBitsReader<'a>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn new` · L289

```rust
pub fn new(input: &'a [u8]) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn from_raw_parts` · L294

```rust
pub unsafe fn from_raw_parts(ptr: *const u8, len: usize) -> Self
```

`ptr` must be valid for `len` readable bytes for `'a`.

### `pub fn position` · L304

```rust
pub fn position(&self) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_bits` · L307

```rust
pub fn read_bits(&mut self, width: u32) -> QslResult<u64>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn align` · L332

```rust
pub fn align(&mut self) -> QslResult<()>
```

Ignore only zero-filled alignment bits.

### `pub fn read_bytes` · L340

```rust
pub fn read_bytes(&mut self, n: usize) -> QslResult<&'a [u8]>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_u8` · L350

```rust
pub fn read_u8(&mut self) -> QslResult<u8>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_u16` · L353

```rust
pub fn read_u16(&mut self) -> QslResult<u16>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_u32` · L356

```rust
pub fn read_u32(&mut self) -> QslResult<u32>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_u64` · L359

```rust
pub fn read_u64(&mut self) -> QslResult<u64>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_i8` · L362

```rust
pub fn read_i8(&mut self) -> QslResult<i8>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_i16` · L365

```rust
pub fn read_i16(&mut self) -> QslResult<i16>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_i32` · L368

```rust
pub fn read_i32(&mut self) -> QslResult<i32>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn read_i64` · L371

```rust
pub fn read_i64(&mut self) -> QslResult<i64>
```

No Rustdoc text adjacent to this declaration in the supplied source.


## `optional.rs`

### `pub struct OptionalOffsets` · L8

```rust
pub struct OptionalOffsets<'a>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn parse` · L14

```rust
pub fn parse(table_and_payload: &'a [u8]) -> QslResult<Self>
```

Check metadata once. This does not validate individual field codecs/UTF-8.

### `pub fn len` · L45

```rust
pub fn len(&self) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn is_empty` · L49

```rust
pub fn is_empty(&self) -> bool
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn offset` · L54

```rust
pub fn offset(&self, index: usize) -> Option<usize>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn field_tail` · L68

```rust
pub fn field_tail(&self, index: usize) -> Option<&'a [u8]>
```

No Rustdoc text adjacent to this declaration in the supplied source.


## `parser.rs`

### `pub struct ParseError` · L12

```rust
pub struct ParseError
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn span` · L26

```rust
pub fn span(&self) -> (usize, usize)
```

Byte offsets are suitable for the existing `qsl_error` source highlighter.

### `pub fn parse_qsl` · L222

```rust
pub fn parse_qsl(source: &str) -> Result<Vec<QslSchema>, ParseError>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn parse_qsl_with_constants` · L226

```rust
pub fn parse_qsl_with_constants( source: &str, globals: &HashMap<String, i128>, ) -> Result<Vec<QslSchema>, ParseError>
```

No Rustdoc text adjacent to this declaration in the supplied source.


## `parser_util.rs`

### `pub struct ParseNum` · L4

```rust
pub struct ParseNum(pub usize, pub bool);
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn rank_bits` · L7

```rust
pub fn rank_bits(min: ParseNum, max: ParseNum) -> Option<usize>
```

Inclusive range width: ceil(log2(MAX-MIN+1)), including singleton = 0.

### `pub fn parse_bytes` · L19

```rust
pub fn parse_bytes(bytes: &[u8]) -> Option<usize>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn parse_hex_bytes` · L33

```rust
pub fn parse_hex_bytes(bytes: &[u8]) -> Result<usize, &'static str>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn scan_schema_blocks` · L55

```rust
pub fn scan_schema_blocks(source: &[u8]) -> Result<Vec<&[u8]>, &'static str>
```

Linear scanner for `&Name{...}` blocks. Cursor and ranges are always absolute indexes into the same source (unlike the original offset mixup).

### `pub fn parse_schema_flags` · L74

```rust
pub fn parse_schema_flags(source: &[u8]) -> Result<usize, &'static str>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn parse_primitive_type` · L96

```rust
pub fn parse_primitive_type(token: &[u8]) -> Option<crate::compat::ParserQslType>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn encode_signed_minimum` · L112

```rust
pub fn encode_signed_minimum(value: ParseNum) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.


## `serialize.rs`

### `pub trait Serializable: Sized` · L3

```rust
pub trait Serializable: Sized
```

Flat, trusted record serialization. No dynamic dispatch, runtime validation or `Result` is part of this API. The caller supplies schema-valid data.

### `fn get_size` · L10

```rust
fn get_size() -> usize;
```

Smallest possible encoded size determined at build time.

### `fn serialized_size` · L15

```rust
fn serialized_size(&self) -> usize;
```

Required writable capacity. Exact for ordinary fields; records with compressed strings return the raw-size upper bound. Use the count from `serialize_into_unsafe`/`serialize_to` as the initialized wire length.

### `fn serialize` · L21

```rust
fn serialize(&self) -> SerOut
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `fn serialize` · L35

```rust
fn serialize(&self) -> Vec<u8>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `unsafe fn serialize_unsafe` · L48

```rust
unsafe fn serialize_unsafe(&self, to: *mut u8);
```

`to` must address `serialized_size()` writable, nonoverlapping bytes. Every field must obey the QSL bounds and fixed byte lengths.

### `unsafe fn serialize_into_unsafe` · L54

```rust
unsafe fn serialize_into_unsafe(&self, to: *mut u8) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `unsafe fn serialize_to` · L63

```rust
unsafe fn serialize_to(&self, to: &mut SerOut) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `unsafe fn serialize_to` · L73

```rust
unsafe fn serialize_to(&self, to: &mut [u8]) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `unsafe fn deserialize` · L80

```rust
unsafe fn deserialize(input: &[u8]) -> Self
```

Convenience pointer-based decoder with a borrowed-slice argument. `input` must contain a complete, valid record matching the schema.

### `unsafe fn deserialize_into` · L89

```rust
unsafe fn deserialize_into(bytes: *const u8, len: usize, out: *mut Self)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `unsafe fn deserialize_unsafe` · L95

```rust
unsafe fn deserialize_unsafe(bytes: *const u8) -> Self;
```

`bytes` must point to a complete, valid schema-encoded record.

### `unsafe fn deserialize_from_raw_parts` · L100

```rust
unsafe fn deserialize_from_raw_parts(bytes: *const u8, _len: usize) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub trait BorrowSerializable` · L107

```rust
pub trait BorrowSerializable<'a>: Sized
```

Zero-copy records borrow their string/byte payloads from the input memory. The backing allocation must remain alive for the complete `'a` lifetime.

### `fn get_size` · L113

```rust
fn get_size() -> usize;
```

Minimum encoded size including required fields and header bytes.

### `fn serialized_size` · L115

```rust
fn serialized_size(&self) -> usize;
```

Required output capacity; compressed strings use their raw-size bound.

### `fn serialize` · L121

```rust
fn serialize(&self) -> SerOut
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `fn serialize` · L133

```rust
fn serialize(&self) -> Vec<u8>
```

Allocate once and encode into bytes initialized by the generated writer. Values must satisfy all QSL bounds and exact byte lengths.

### `unsafe fn serialize_unsafe` · L145

```rust
unsafe fn serialize_unsafe(&self, to: *mut u8);
```

`to` must address `self.serialized_size()` exclusively writable bytes. Every field must obey the QSL bounds and fixed byte lengths.

### `unsafe fn serialize_into_unsafe` · L151

```rust
unsafe fn serialize_into_unsafe(&self, to: *mut u8) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `unsafe fn serialize_to` · L159

```rust
unsafe fn serialize_to(&self, to: &mut SerOut) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `unsafe fn serialize_to` · L168

```rust
unsafe fn serialize_to(&self, to: &mut [u8]) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `unsafe fn deserialize_borrowed` · L174

```rust
unsafe fn deserialize_borrowed(input: &'a [u8]) -> Self
```

`input` must be a complete, valid QSL record for the lifetime `'a`.

### `unsafe fn deserialize_borrowed_into` · L183

```rust
unsafe fn deserialize_borrowed_into(bytes: *const u8, len: usize, out: *mut Self)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `unsafe fn deserialize_borrowed_unsafe` · L190

```rust
unsafe fn deserialize_borrowed_unsafe(bytes: *const u8) -> Self;
```

Caller must guarantee that `bytes` points to a complete valid record and remains alive for `'a`.

### `unsafe fn deserialize_borrowed_from_raw_parts` · L195

```rust
unsafe fn deserialize_borrowed_from_raw_parts(bytes: *const u8, _len: usize) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub struct SerOut` · L201

```rust
pub struct SerOut
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn new` · L210

```rust
pub fn new(len: usize) -> Self
```

Allocate initialized zero-filled bytes using the global Rust allocator.

### `pub fn with_capacity_aligned` · L229

```rust
pub fn with_capacity_aligned(capacity: usize, alignment: usize) -> Self
```

Allocate reusable storage with architecture-selected power-of-two alignment. The initialized slice is initially empty; capacity is retained after reuse.

### `pub fn capacity` · L242

```rust
pub fn capacity(&self) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn alignment` · L246

```rust
pub fn alignment(&self) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn clear` · L250

```rust
pub fn clear(&mut self)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn set_len` · L257

```rust
pub unsafe fn set_len(&mut self, len: usize)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn from_raw` · L263

```rust
pub unsafe fn from_raw(ptr: *mut u8, layout: std::alloc::Layout, len: usize) -> Self
```

Takes ownership of a caller allocation with the exact given layout. Must point to an allocation obtained with the same layout and allocator.

### `pub fn as_mut_ptr` · L272

```rust
pub fn as_mut_ptr(&mut self) -> *mut u8
```

Return the stable output allocation pointer; does not advance a cursor.


## `socket.rs`

### `pub struct RawSocket` · L10

```rust
pub struct RawSocket
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn new` · L15

```rust
pub fn new(stream: TcpStream, max_frame: usize) -> io::Result<Self>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn receive` · L34

```rust
pub fn receive<'a>(&mut self, storage: &'a mut [u8]) -> io::Result<&'a [u8]>
```

Read exactly one bounded frame into caller-owned storage. An oversized frame terminates this stream to avoid losing framing state.

### `pub unsafe fn send_record` · L74

```rust
pub unsafe fn send_record<T: Serializable>( &mut self, value: &T, storage: &mut [u8], ) -> io::Result<usize>
```

Serialize directly after the transport prefix and send the initialized bytes. Every record field must satisfy its schema's range and UTF-8 contracts.

### `pub unsafe fn send_borrowed_record` · L85

```rust
pub unsafe fn send_borrowed_record<'a, T: BorrowSerializable<'a>>( &mut self, value: &T, storage: &mut [u8], ) -> io::Result<usize>
```

Every record field must satisfy its schema's range and UTF-8 contracts.

### `pub fn stream` · L94

```rust
pub fn stream(&self) -> &TcpStream
```

No Rustdoc text adjacent to this declaration in the supplied source.


## `strings.rs`

### `pub const STRING_COMPRESSION_MIN_BYTES: usize = 32;` · L12

```rust
pub const STRING_COMPRESSION_MIN_BYTES: usize = 32;
```

Strings shorter than this are copied raw, without a compression probe.

### `pub struct InlineString` · L22

```rust
pub struct InlineString<const MAX: usize>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub const MAX_BYTES: usize = MAX;` · L29

```rust
pub const MAX_BYTES: usize = MAX;
```

Maximum decoded UTF-8 byte count, fixed by the schema.

### `pub const HEADER_BYTES: usize = if MAX` · L31

```rust
pub const HEADER_BYTES: usize = if MAX <= 127
```

Length and mode share a word: 1 byte through MAX=127, then 2 or 4.

### `pub const OFFSET_BYTES: usize = if MAX` · L39

```rust
pub const OFFSET_BYTES: usize = if MAX <= 255
```

Width of each backward distance; match lengths use the control byte.

### `pub const fn new` · L44

```rust
pub const fn new() -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn from_str_unchecked` · L56

```rust
pub unsafe fn from_str_unchecked(value: &str) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub const fn spaces` · L70

```rust
pub const fn spaces<const LEN: usize>() -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn len` · L84

```rust
pub fn len(&self) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn is_empty` · L90

```rust
pub fn is_empty(&self) -> bool
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn as_str` · L96

```rust
pub fn as_str(&self) -> &str
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn wire_capacity` · L108

```rust
pub fn wire_capacity(&self) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn encode_to` · L117

```rust
pub unsafe fn encode_to(&self, output: *mut u8) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn encode_raw_to` · L125

```rust
pub unsafe fn encode_raw_to(&self, output: *mut u8) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn decode_into` · L140

```rust
pub unsafe fn decode_into(input: *const u8, output: *mut Self) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn decode_raw_into` · L150

```rust
pub unsafe fn decode_raw_into(input: *const u8, output: *mut Self) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn decode_from` · L167

```rust
pub unsafe fn decode_from(input: *const u8) -> (Self, usize)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn encode_bounded` · L319

```rust
pub unsafe fn encode_bounded<const MAX: usize>( input: *const u8, len: usize, output: *mut u8, ) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn decode_bounded` · L577

```rust
pub unsafe fn decode_bounded<const MAX: usize>( input: *const u8, output: *mut u8, ) -> (usize, usize)
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub const FIXED_UTF8_CODEC_VERSION: u8 = 2;` · L662

```rust
pub const FIXED_UTF8_CODEC_VERSION: u8 = 2;
```

Fixed UTF-8 framing revision; peers must use the same revision.

### `pub unsafe fn encode_fixed_utf8` · L670

```rust
pub unsafe fn encode_fixed_utf8<const MAX: usize>(input: *const u8, output: *mut u8) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub unsafe fn decode_fixed_utf8` · L692

```rust
pub unsafe fn decode_fixed_utf8<const MAX: usize>(input: *const u8, output: *mut u8) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub struct Utf8Guard` · L714

```rust
pub struct Utf8Guard<'a>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub const fn from_str` · L719

```rust
pub const fn from_str(value: &'a str) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn try_from_bytes` · L725

```rust
pub fn try_from_bytes(bytes: &'a [u8]) -> Result<Self, str::Utf8Error>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub const unsafe fn from_bytes_unchecked` · L732

```rust
pub const unsafe fn from_bytes_unchecked(bytes: &'a [u8]) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub const fn as_bytes` · L736

```rust
pub const fn as_bytes(self) -> &'a [u8]
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub const fn as_str` · L740

```rust
pub const fn as_str(self) -> &'a str
```

No Rustdoc text adjacent to this declaration in the supplied source.


## `types.rs`

### `pub const FLAG_COMPACT: usize = 0b0000_0001;` · L4

```rust
pub const FLAG_COMPACT: usize = 0b0000_0001;
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub const FLAG_SAFETY: usize = 0b0000_0010;` · L5

```rust
pub const FLAG_SAFETY: usize = 0b0000_0010;
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub const FLAG_ALIGNMENT: usize = 0b0000_0100;` · L6

```rust
pub const FLAG_ALIGNMENT: usize = 0b0000_0100;
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub const FLAG_OPTIONAL_BITMAP: usize = 0b0001_0000;` · L7

```rust
pub const FLAG_OPTIONAL_BITMAP: usize = 0b0001_0000;
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub const FLAG_OPT_TABLE: usize = 0b0010_0000;` · L8

```rust
pub const FLAG_OPT_TABLE: usize = 0b0010_0000;
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub const FLAG_PREDICT_SEQ: usize = 0b0100_0000;` · L9

```rust
pub const FLAG_PREDICT_SEQ: usize = 0b0100_0000;
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub const FLAG_FIELD_BORROW: usize = 0b1000_0000;` · L11

```rust
pub const FLAG_FIELD_BORROW: usize = 0b1000_0000;
```

Internal lifetime marker for per-field byte/guard borrows.

### `pub const FLAG_LIFETIME: usize = 0b0000_1000;` · L12

```rust
pub const FLAG_LIFETIME: usize = 0b0000_1000;
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub trait QslCustom: Sized` · L19

```rust
pub trait QslCustom: Sized
```

Compile-time-sized custom codec used by generated, unchecked QSL I/O.  All methods are inlined into generated serializers. No `Result` or buffer-length bookkeeping is required at runtime. Implementors must obey the exact wire-size contract under both normal and compact encodings.

### `unsafe fn qsl_write_unchecked` · L29

```rust
unsafe fn qsl_write_unchecked(&self, to: *mut u8);
```

Constant-size normal codec. Use a raw pointer store only. `to` must point to `Self::SIZE` exclusively writable bytes.

### `unsafe fn qsl_read_unchecked` · L34

```rust
unsafe fn qsl_read_unchecked(from: *const u8) -> Self;
```

Read an owned value without checking its input buffer. `from` must point to `Self::SIZE` readable, schema-valid bytes.

### `fn qsl_size` · L38

```rust
fn qsl_size(&self) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `fn qsl_comp_size` · L44

```rust
fn qsl_comp_size(&self) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `unsafe fn qsl_comp_write_unchecked` · L51

```rust
unsafe fn qsl_comp_write_unchecked(&self, to: *mut u8)
```

Compact raw-pointer encoder (defaults to the normal encoder). `to` must have `Self::COMP_SIZE` valid writable bytes.

### `unsafe fn qsl_comp_read_unchecked` · L58

```rust
unsafe fn qsl_comp_read_unchecked(from: *const u8) -> Self
```

Compact raw-pointer decoder (defaults to the normal decoder). `from` must have `Self::COMP_SIZE` valid readable bytes.

### `pub enum QslType` · L64

```rust
pub enum QslType
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn rank_bits` · L128

```rust
pub fn rank_bits(min: i128, max: i128) -> QslResult<usize>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn rank_width` · L138

```rust
pub fn rank_width(&self, fallback: usize) -> QslResult<usize>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn rank_base` · L148

```rust
pub fn rank_base(&self) -> i128
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn number16` · L159

```rust
pub fn number16(signed: bool, min: usize) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn number32` · L166

```rust
pub fn number32(signed: bool, min: usize) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn number64` · L173

```rust
pub fn number64(signed: bool, min: usize) -> Self
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn primitive` · L180

```rust
pub fn primitive(&self) -> QslResult<Option<&'static str>>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn rust_type` · L208

```rust
pub fn rust_type(&self, borrowed: bool) -> QslResult<String>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn fixed_wire_size` · L239

```rust
pub fn fixed_wire_size(&self) -> QslResult<Option<usize>>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn default_compact_bits` · L277

```rust
pub fn default_compact_bits(&self) -> QslResult<usize>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn validate` · L295

```rust
pub fn validate(&self, borrowed: bool) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub struct QslField` · L347

```rust
pub struct QslField
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub struct QslSchema` · L361

```rust
pub struct QslSchema
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn compact` · L369

```rust
pub fn compact(&self) -> bool
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn bitmap` · L372

```rust
pub fn bitmap(&self) -> bool
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn opt_table` · L375

```rust
pub fn opt_table(&self) -> bool
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn predict_seq` · L378

```rust
pub fn predict_seq(&self) -> bool
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn sequential` · L382

```rust
pub fn sequential(&self) -> bool
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn lifetime` · L385

```rust
pub fn lifetime(&self) -> bool
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn borrow_strings` · L388

```rust
pub fn borrow_strings(&self) -> bool
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn optional_count` · L391

```rust
pub fn optional_count(&self) -> usize
```

No Rustdoc text adjacent to this declaration in the supplied source.

### `pub fn tag_bits` · L395

```rust
pub fn tag_bits(&self) -> usize
```

Example: max_tag=8 => exactly 3 bits, storing tag-1.

### `pub fn count_bits` · L403

```rust
pub fn count_bits(&self) -> usize
```

Number of bits needed to represent 0..=optional_count.

### `pub fn bitmap_words` · L412

```rust
pub fn bitmap_words(&self) -> usize
```

One presence bit for every declared field, including required fields.

### `pub fn validate` · L415

```rust
pub fn validate(&self) -> QslResult<()>
```

No Rustdoc text adjacent to this declaration in the supplied source.

