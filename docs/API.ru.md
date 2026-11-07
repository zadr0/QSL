# Индекс публичных объявлений

[English](API.en.md) · [Архитектура](architecture.ru.md)

Найдено 756 явно публичных объявлений и сигнатур методов trait в текущих исходниках. Это source-index, а не отражение всех платформенных export-символов. Учитывайте feature rust-macros/diagnostics; native/Windows зависимость удалена; кодогенераторские строковые литералы не считаются API. Пути и строки относятся к локальному снимку из [manifest](../reference/crates-release-0.2.0/source-manifest.json). Репозиторий содержит самостоятельные Cargo-пакеты и документацию QSL 0.2.0.

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

`qsl-interpreter/src/interpreter.rs:173`

```rust
pub trait TypeInterpreter {
```

`qsl-interpreter/src/interpreter.rs:174`

```rust
fn type_name(&self, out: &mut Vec<u8>, borrowed: bool) -> QslResult<()>;
```

`qsl-interpreter/src/interpreter.rs:175`

```rust
fn size(&self, out: &mut Vec<u8>, expr: &str, width: usize, compact: bool) -> QslResult<()>;
```

`qsl-interpreter/src/interpreter.rs:176`

```rust
fn write(&self, out: &mut Vec<u8>, expr: &str) -> QslResult<()>;
```

`qsl-interpreter/src/interpreter.rs:177`

```rust
fn read(&self, out: &mut Vec<u8>, name: &str, borrowed: bool) -> QslResult<()>;
```

`qsl-interpreter/src/interpreter.rs:178`

```rust
fn comp_write(&self, out: &mut Vec<u8>, expr: &str, bits: usize) -> QslResult<()>;
```

`qsl-interpreter/src/interpreter.rs:179`

```rust
fn comp_read( &self, out: &mut Vec<u8>, name: &str, bits: usize, borrowed: bool, ) -> QslResult<()>;
```

`qsl-interpreter/src/interpreter.rs:2691`

```rust
pub struct QslInterpreter {
```

`qsl-interpreter/src/interpreter.rs:2697`

```rust
pub type RustGenerator = QslInterpreter;
```

`qsl-interpreter/src/interpreter.rs:2722`

```rust
pub fn new() -> Self {
```

`qsl-interpreter/src/interpreter.rs:2727`

```rust
pub fn generate(&mut self, schemas: &[QslSchema]) -> QslResult<()> {
```

`qsl-interpreter/src/interpreter.rs:2735`

```rust
pub fn generate_analyzed(&mut self, module: &crate::AnalyzedModule) -> QslResult<()> {
```

`qsl-interpreter/src/interpreter.rs:2742`

```rust
pub fn generate_prepared(&mut self, module: &crate::frontend::PreparedModule) -> QslResult<()> {
```

`qsl-interpreter/src/interpreter.rs:2786`

```rust
pub fn generate_parsed(&mut self, schemas: &[crate::compat::ParserQslSchema]) -> QslResult<()> {
```

`qsl-interpreter/src/interpreter.rs:2794`

```rust
pub fn generate_field(&mut self, field: &QslField, borrowed: bool) -> QslResult<()> {
```

`qsl-interpreter/src/interpreter.rs:3218`

```rust
pub fn generate_default(&mut self, schema: &QslSchema) -> QslResult<()> {
```

`qsl-interpreter/src/interpreter.rs:3236`

```rust
pub fn generate_lifetime(&mut self, schema: &QslSchema) -> QslResult<()> {
```

`qsl-interpreter/src/interpreter.rs:3242`

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

`qsl-lexer/src/analysis.rs:5`

```rust
pub enum BitRegister {
```

`qsl-lexer/src/analysis.rs:14`

```rust
pub fn for_bits(bits: usize) -> QslResult<Self> {
```

`qsl-lexer/src/analysis.rs:25`

```rust
pub fn bits(self) -> usize {
```

`qsl-lexer/src/analysis.rs:35`

```rust
pub fn reader(self) -> &'static str {
```

`qsl-lexer/src/analysis.rs:45`

```rust
pub fn writer(self) -> &'static str {
```

`qsl-lexer/src/analysis.rs:57`

```rust
pub struct BitBlock {
```

`qsl-lexer/src/analysis.rs:62`

```rust
pub struct QslIoPlan {
```

`qsl-lexer/src/analysis.rs:111`

```rust
pub fn analyze(schema: &QslSchema) -> QslResult<Self> {
```

## qsl-lexer/src/bitmap.rs

`qsl-lexer/src/bitmap.rs:4`

```rust
pub struct BitmapPlan {
```

`qsl-lexer/src/bitmap.rs:9`

```rust
pub fn new(fields: usize) -> QslResult<Self> {
```

`qsl-lexer/src/bitmap.rs:21`

```rust
pub fn bytes(self) -> usize {
```

`qsl-lexer/src/bitmap.rs:24`

```rust
pub fn rust_type(self) -> &'static str {
```

`qsl-lexer/src/bitmap.rs:34`

```rust
pub fn marker_bit(self, marker: usize) -> QslResult<u128> {
```

`qsl-lexer/src/bitmap.rs:40`

```rust
pub fn validate(self, mask: u128) -> QslResult<()> {
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

`qsl-lexer/src/config.rs:4`

```rust
pub const DEFAULT_MAX_DATA_SIZE: usize = 65535;
```

`qsl-lexer/src/config.rs:7`

```rust
pub struct GenerationConfig {
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

`qsl-lexer/src/hybrid.rs:4`

```rust
pub struct HybridPlan {
```

`qsl-lexer/src/hybrid.rs:13`

```rust
pub fn analyze(s: &QslSchema) -> QslResult<Option<Self>> {
```

## qsl-lexer/src/io.rs

`qsl-lexer/src/io.rs:1`

```rust
pub type QslResult<T> = Result<T, QslError>;
```

`qsl-lexer/src/io.rs:4`

```rust
pub enum QslError {
```

`qsl-lexer/src/io.rs:27`

```rust
pub struct QslSizer {
```

`qsl-lexer/src/io.rs:34`

```rust
pub fn bits(&mut self, n: usize) -> QslResult<()> {
```

`qsl-lexer/src/io.rs:44`

```rust
pub fn align(&mut self) -> QslResult<()> {
```

`qsl-lexer/src/io.rs:52`

```rust
pub fn bytes(&mut self, n: usize) -> QslResult<()> {
```

`qsl-lexer/src/io.rs:58`

```rust
pub fn finish(mut self) -> QslResult<usize> {
```

## qsl-lexer/src/length_graph.rs

`qsl-lexer/src/length_graph.rs:6`

```rust
pub struct Domain {
```

`qsl-lexer/src/length_graph.rs:13`

```rust
pub struct LengthNode {
```

`qsl-lexer/src/length_graph.rs:22`

```rust
pub struct OffsetRecipe {
```

`qsl-lexer/src/length_graph.rs:29`

```rust
pub struct LengthGraph {
```

`qsl-lexer/src/length_graph.rs:37`

```rust
pub fn analyze(s: &QslSchema) -> QslResult<Option<Self>> {
```

`qsl-lexer/src/length_graph.rs:219`

```rust
pub fn event_index(&self, len: usize) -> Option<usize> {
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

## qsl-lexer/src/modernization.rs

`qsl-lexer/src/modernization.rs:5`

```rust
pub struct SchemaOptions {
```

`qsl-lexer/src/modernization.rs:21`

```rust
pub fn with_native_overlay(mut self) -> Self {
```

`qsl-lexer/src/modernization.rs:26`

```rust
pub fn native_overlay(&self) -> bool {
```

`qsl-lexer/src/modernization.rs:34`

```rust
pub fn register_word(bytes: usize) -> QslResult<&'static str> {
```

`qsl-lexer/src/modernization.rs:46`

```rust
pub struct RegisterGroup {
```

`qsl-lexer/src/modernization.rs:54`

```rust
pub fn register_groups(s: &QslSchema) -> QslResult<Vec<RegisterGroup>> {
```

`qsl-lexer/src/modernization.rs:135`

```rust
pub enum StorageKind {
```

`qsl-lexer/src/modernization.rs:145`

```rust
pub fn field_rust_type(f: &crate::QslField, borrowed: bool) -> QslResult<String> {
```

`qsl-lexer/src/modernization.rs:172`

```rust
pub fn lower_storage(s: &QslSchema, limit: usize) -> QslResult<QslSchema> {
```

`qsl-lexer/src/modernization.rs:252`

```rust
pub fn validate_options(s: &QslSchema) -> Result<(), String> {
```

`qsl-lexer/src/modernization.rs:383`

```rust
pub struct NativeRegion {
```

`qsl-lexer/src/modernization.rs:414`

```rust
pub fn check_native_offsets(s: &QslSchema) -> Result<(), String> {
```

`qsl-lexer/src/modernization.rs:450`

```rust
pub struct OptionalRepresentation {
```

`qsl-lexer/src/modernization.rs:457`

```rust
pub fn optional_representation( f: &crate::QslField, aggressive: bool, ) -> Option<OptionalRepresentation> {
```

`qsl-lexer/src/modernization.rs:515`

```rust
pub fn check_storage_limit(s: &QslSchema, limit: usize) -> Result<(), String> {
```

`qsl-lexer/src/modernization.rs:556`

```rust
pub struct NativeTypeLayout {
```

`qsl-lexer/src/modernization.rs:564`

```rust
pub fn native_type_layout( t: &QslType, types: &std::collections::BTreeMap<String, NativeTypeLayout>, ) -> Option<NativeTypeLayout> {
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

`qsl-lexer/src/payload_quant.rs:5`

```rust
pub struct QuantField {
```

`qsl-lexer/src/payload_quant.rs:17`

```rust
pub struct PayloadQuantPlan {
```

`qsl-lexer/src/payload_quant.rs:29`

```rust
pub fn analyze(s: &QslSchema) -> QslResult<Self> {
```

## qsl-lexer/src/primitive_macro.rs

`qsl-lexer/src/primitive_macro.rs:26`

```rust
macro_rules! qsl_primitive { ... }
```

`qsl-lexer/src/primitive_macro.rs:92`

```rust
macro_rules! qsl_native_primitive { ... }
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

`qsl-lexer/src/rust_macro.rs:226`

```rust
pub fn parse_rust_macros(sources: &[&str]) -> Result<Vec<QslSchema>, MacroError> {
```

`qsl-lexer/src/rust_macro.rs:365`

```rust
pub fn expand_with_rust( schemas: &[QslSchema], sources: &[&str], ) -> Result<Vec<QslSchema>, MacroError> {
```

`qsl-lexer/src/rust_macro.rs:407`

```rust
pub fn parse_qsl_with_rust( source: &str, constants: &HashMap<String, i128>, rust_sources: &[&str], ) -> Result<Vec<QslSchema>, MacroError> {
```

## qsl-lexer/src/schema_hash.rs

`qsl-lexer/src/schema_hash.rs:7`

```rust
pub const SCHEMA_HASH_VERSION: u16 = 1;
```

`qsl-lexer/src/schema_hash.rs:11`

```rust
pub enum SchemaHashAlgorithm {
```

`qsl-lexer/src/schema_hash.rs:19`

```rust
pub const fn bits(self) -> u32 {
```

`qsl-lexer/src/schema_hash.rs:25`

```rust
pub const fn name(self) -> &'static str {
```

`qsl-lexer/src/schema_hash.rs:31`

```rust
pub const fn rust_variant(self) -> &'static str {
```

`qsl-lexer/src/schema_hash.rs:56`

```rust
pub enum SchemaHashKind {
```

`qsl-lexer/src/schema_hash.rs:65`

```rust
pub struct SchemaHashes {
```

`qsl-lexer/src/schema_hash.rs:75`

```rust
pub fn strict_hex(self) -> String {
```

`qsl-lexer/src/schema_hash.rs:78`

```rust
pub fn light_hex(self) -> String {
```

`qsl-lexer/src/schema_hash.rs:90`

```rust
pub trait SchemaIdentity {
```

`qsl-lexer/src/schema_hash.rs:97`

```rust
pub fn hashes(&self, algorithm: SchemaHashAlgorithm) -> QslResult<SchemaHashes> {
```

`qsl-lexer/src/schema_hash.rs:112`

```rust
pub fn hash_descriptor(&self, kind: SchemaHashKind) -> QslResult<Vec<u8>> {
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

`qsl-lexer/src/static_index.rs:6`

```rust
pub const ANALYSIS_MAX_STATES: usize = 4096;
```

`qsl-lexer/src/static_index.rs:7`

```rust
pub const ANALYSIS_MAX_BYTES: usize = 65535;
```

`qsl-lexer/src/static_index.rs:10`

```rust
pub struct StateCode {
```

`qsl-lexer/src/static_index.rs:18`

```rust
pub struct OffsetRow {
```

`qsl-lexer/src/static_index.rs:24`

```rust
pub struct PredictPlan {
```

`qsl-lexer/src/static_index.rs:34`

```rust
pub fn analyze(schema: &QslSchema) -> QslResult<Self> {
```

`qsl-lexer/src/static_index.rs:149`

```rust
pub fn packed_index(&self) -> QslResult<Vec<u32>> {
```

`qsl-lexer/src/static_index.rs:153`

```rust
pub fn pack_lengths(&self, lengths: &[Option<usize>]) -> QslResult<u32> {
```

`qsl-lexer/src/static_index.rs:169`

```rust
pub fn unpack_lengths(&self, d: u32) -> QslResult<Vec<Option<usize>>> {
```

`qsl-lexer/src/static_index.rs:201`

```rust
pub fn quantize_index(index: &[u32], quantum: usize) -> QslResult<Vec<u32>> {
```

`qsl-lexer/src/static_index.rs:229`

```rust
pub fn lookup_index(table: &[u32], quantum: usize, index: usize) -> Option<u32> {
```

`qsl-lexer/src/static_index.rs:243`

```rust
pub fn structural_counts(domains: &[Vec<usize>], max_len: usize) -> QslResult<Vec<u64>> {
```

## qsl-lexer/src/static_optional.rs

`qsl-lexer/src/static_optional.rs:4`

```rust
pub enum StaticPresence {
```

`qsl-lexer/src/static_optional.rs:11`

```rust
pub struct StaticOptionalPlan {
```

`qsl-lexer/src/static_optional.rs:16`

```rust
pub fn analyze(schema: &QslSchema) -> QslResult<Self> {
```

`qsl-lexer/src/static_optional.rs:75`

```rust
pub fn header_bytes(&self) -> usize {
```

## qsl-lexer/src/types.rs

`qsl-lexer/src/types.rs:4`

```rust
pub const FLAG_COMPACT: usize = 0b0000_0001;
```

`qsl-lexer/src/types.rs:5`

```rust
pub const FLAG_SAFETY: usize = 0b0000_0010;
```

`qsl-lexer/src/types.rs:6`

```rust
pub const FLAG_ALIGNMENT: usize = 0b0000_0100;
```

`qsl-lexer/src/types.rs:7`

```rust
pub const FLAG_OPTIONAL_BITMAP: usize = 0b0001_0000;
```

`qsl-lexer/src/types.rs:8`

```rust
pub const FLAG_OPT_TABLE: usize = 0b0010_0000;
```

`qsl-lexer/src/types.rs:9`

```rust
pub const FLAG_PREDICT_SEQ: usize = 0b0100_0000;
```

`qsl-lexer/src/types.rs:11`

```rust
pub const FLAG_FIELD_BORROW: usize = 0b1000_0000;
```

`qsl-lexer/src/types.rs:12`

```rust
pub const FLAG_LIFETIME: usize = 0b0000_1000;
```

`qsl-lexer/src/types.rs:13`

```rust
pub const FLAG_STATE_BITS: usize = 1 << 8;
```

`qsl-lexer/src/types.rs:14`

```rust
pub const FLAG_INLINE: usize = 1 << 9;
```

`qsl-lexer/src/types.rs:15`

```rust
pub const FLAG_INLINE_ALWAYS: usize = 1 << 10;
```

`qsl-lexer/src/types.rs:16`

```rust
pub const FLAG_INLINE_NEVER: usize = 1 << 11;
```

`qsl-lexer/src/types.rs:17`

```rust
pub const STATE_BITS_QUANTUM_SHIFT: usize = 12;
```

`qsl-lexer/src/types.rs:18`

```rust
pub const STATE_BITS_QUANTUM_MASK: usize = 7 << STATE_BITS_QUANTUM_SHIFT;
```

`qsl-lexer/src/types.rs:20`

```rust
pub const FLAG_MACRO: usize = 1 << 15;
```

`qsl-lexer/src/types.rs:22`

```rust
pub const FLAG_OPT_TABLE_LEGACY: usize = 1 << 16;
```

`qsl-lexer/src/types.rs:23`

```rust
pub const FLAG_PAYLOAD_QUANT: usize = 1 << 17;
```

`qsl-lexer/src/types.rs:25`

```rust
pub const FLAG_QPS: usize = FLAG_PAYLOAD_QUANT;
```

`qsl-lexer/src/types.rs:26`

```rust
pub const PAYLOAD_QUANT_SHIFT: usize = 18;
```

`qsl-lexer/src/types.rs:27`

```rust
pub const PAYLOAD_QUANT_MASK: usize = 3 << PAYLOAD_QUANT_SHIFT;
```

`qsl-lexer/src/types.rs:34`

```rust
pub trait QslCustom: Sized {
```

`qsl-lexer/src/types.rs:43`

```rust
fn qsl_validate(_bytes: &[u8], _compact: bool) -> QslResult<()> {
```

`qsl-lexer/src/types.rs:49`

```rust
fn qsl_validate_values(&self) -> QslResult<()> {
```

`qsl-lexer/src/types.rs:56`

```rust
unsafe fn qsl_write_unchecked(&self, to: *mut u8);
```

`qsl-lexer/src/types.rs:61`

```rust
unsafe fn qsl_read_unchecked(from: *const u8) -> Self;
```

`qsl-lexer/src/types.rs:65`

```rust
fn qsl_size(&self) -> usize {
```

`qsl-lexer/src/types.rs:71`

```rust
fn qsl_comp_size(&self) -> usize {
```

`qsl-lexer/src/types.rs:78`

```rust
unsafe fn qsl_comp_write_unchecked(&self, to: *mut u8) {
```

`qsl-lexer/src/types.rs:85`

```rust
unsafe fn qsl_comp_read_unchecked(from: *const u8) -> Self {
```

`qsl-lexer/src/types.rs:91`

```rust
pub trait Sealed {
```

`qsl-lexer/src/types.rs:97`

```rust
pub trait PrimitiveInteger: primitive_integer_seal::Sealed {
```

`qsl-lexer/src/types.rs:103`

```rust
pub struct PrimitiveRepr {
```

`qsl-lexer/src/types.rs:110`

```rust
pub fn data(self) -> QslType {
```

`qsl-lexer/src/types.rs:118`

```rust
pub fn validate(self) -> QslResult<()> {
```

`qsl-lexer/src/types.rs:128`

```rust
pub unsafe trait QslPrimitive: QslCustom {
```

`qsl-lexer/src/types.rs:130`

```rust
fn qsl_to_bits(&self) -> u64;
```

`qsl-lexer/src/types.rs:133`

```rust
fn qsl_from_bits(bits: u64) -> Self;
```

`qsl-lexer/src/types.rs:137`

```rust
pub enum QslType {
```

`qsl-lexer/src/types.rs:206`

```rust
pub fn rank_bits(min: i128, max: i128) -> QslResult<usize> {
```

`qsl-lexer/src/types.rs:216`

```rust
pub fn rank_width(&self, fallback: usize) -> QslResult<usize> {
```

`qsl-lexer/src/types.rs:226`

```rust
pub fn rank_base(&self) -> i128 {
```

`qsl-lexer/src/types.rs:238`

```rust
pub fn number16(signed: bool, min: usize) -> Self {
```

`qsl-lexer/src/types.rs:245`

```rust
pub fn number32(signed: bool, min: usize) -> Self {
```

`qsl-lexer/src/types.rs:252`

```rust
pub fn number64(signed: bool, min: usize) -> Self {
```

`qsl-lexer/src/types.rs:259`

```rust
pub fn primitive(&self) -> QslResult<Option<&'static str>> {
```

`qsl-lexer/src/types.rs:287`

```rust
pub fn rust_type(&self, borrowed: bool) -> QslResult<String> {
```

`qsl-lexer/src/types.rs:319`

```rust
pub fn fixed_wire_size(&self) -> QslResult<Option<usize>> {
```

`qsl-lexer/src/types.rs:358`

```rust
pub fn default_compact_bits(&self) -> QslResult<usize> {
```

`qsl-lexer/src/types.rs:376`

```rust
pub fn validate(&self, borrowed: bool) -> QslResult<()> {
```

`qsl-lexer/src/types.rs:431`

```rust
pub struct QslField {
```

`qsl-lexer/src/types.rs:446`

```rust
pub struct QslSchema {
```

`qsl-lexer/src/types.rs:455`

```rust
pub fn compact(&self) -> bool {
```

`qsl-lexer/src/types.rs:458`

```rust
pub fn bitmap(&self) -> bool {
```

`qsl-lexer/src/types.rs:461`

```rust
pub fn opt_table(&self) -> bool {
```

`qsl-lexer/src/types.rs:465`

```rust
pub fn qps(&self) -> bool {
```

`qsl-lexer/src/types.rs:468`

```rust
pub fn quantum(&self) -> usize {
```

`qsl-lexer/src/types.rs:471`

```rust
pub fn quant_requested(&self) -> bool {
```

`qsl-lexer/src/types.rs:474`

```rust
pub fn payload_quantum(&self) -> usize {
```

`qsl-lexer/src/types.rs:477`

```rust
pub fn legacy_opt_table(&self) -> bool {
```

`qsl-lexer/src/types.rs:480`

```rust
pub fn predict_seq(&self) -> bool {
```

`qsl-lexer/src/types.rs:486`

```rust
pub fn state_bits(&self) -> bool {
```

`qsl-lexer/src/types.rs:489`

```rust
pub fn state_bits_quantum(&self) -> usize {
```

`qsl-lexer/src/types.rs:492`

```rust
pub fn predict_table(&self) -> bool {
```

`qsl-lexer/src/types.rs:496`

```rust
pub fn predicted_opt_table(&self) -> bool {
```

`qsl-lexer/src/types.rs:499`

```rust
pub fn sequential(&self) -> bool {
```

`qsl-lexer/src/types.rs:503`

```rust
pub fn lifetime(&self) -> bool {
```

`qsl-lexer/src/types.rs:506`

```rust
pub fn borrow_strings(&self) -> bool {
```

`qsl-lexer/src/types.rs:509`

```rust
pub fn optional_count(&self) -> usize {
```

`qsl-lexer/src/types.rs:513`

```rust
pub fn tag_bits(&self) -> usize {
```

`qsl-lexer/src/types.rs:521`

```rust
pub fn count_bits(&self) -> usize {
```

`qsl-lexer/src/types.rs:530`

```rust
pub fn bitmap_width(&self) -> usize {
```

`qsl-lexer/src/types.rs:533`

```rust
pub fn bitmap_bytes(&self) -> usize {
```

`qsl-lexer/src/types.rs:537`

```rust
pub fn bitmap_words(&self) -> usize {
```

`qsl-lexer/src/types.rs:540`

```rust
pub fn optional_tag_base(&self) -> usize {
```

`qsl-lexer/src/types.rs:546`

```rust
pub fn validate(&self) -> QslResult<()> {
```

`qsl-lexer/src/types.rs:732`

```rust
pub fn apply_primitive_bindings( schema: &mut QslSchema, bindings: &std::collections::BTreeMap<String, PrimitiveRepr>, ) -> QslResult<()> {
```

## qsl-lexer/src/wire.rs

`qsl-lexer/src/wire.rs:3`

```rust
pub const STRING_COMPRESSION_MIN_BYTES: usize = 32;
```

`qsl-lexer/src/wire.rs:5`

```rust
pub const STRING_ENCODER_REVISION: u8 = 2;
```

`qsl-lexer/src/wire.rs:8`

```rust
pub const fn inline_string_header_bytes(max: usize) -> usize {
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

`qsl/src/io.rs:7`

```rust
pub use checked::{QslCheckedBitsReader, QslCheckedBitsWriter, QslCheckedReader, QslCheckedWriter};
```

`qsl/src/io.rs:9`

```rust
pub use qsl_lexer::io::{QslError, QslResult, QslSizer};
```

`qsl/src/io.rs:14`

```rust
pub struct QslRawWriter {
```

`qsl/src/io.rs:22`

```rust
pub unsafe fn new(ptr: *mut u8) -> Self {
```

`qsl/src/io.rs:29`

```rust
pub fn from(out: &mut [u8]) -> Self {
```

`qsl/src/io.rs:39`

```rust
pub unsafe fn write_u8(&mut self, value: u8) {
```

`qsl/src/io.rs:47`

```rust
pub unsafe fn write_u16(&mut self, value: u16) {
```

`qsl/src/io.rs:55`

```rust
pub unsafe fn write_u32(&mut self, value: u32) {
```

`qsl/src/io.rs:63`

```rust
pub unsafe fn write_u64(&mut self, value: u64) {
```

`qsl/src/io.rs:71`

```rust
pub unsafe fn write(&mut self, source: *const u8, len: usize) {
```

`qsl/src/io.rs:79`

```rust
pub struct QslRawReader {
```

`qsl/src/io.rs:87`

```rust
pub unsafe fn new(ptr: *const u8) -> Self {
```

`qsl/src/io.rs:93`

```rust
pub fn from(input: &[u8]) -> Self {
```

`qsl/src/io.rs:102`

```rust
pub unsafe fn read_u8(&mut self) -> u8 {
```

`qsl/src/io.rs:111`

```rust
pub unsafe fn read_u16(&mut self) -> u16 {
```

`qsl/src/io.rs:120`

```rust
pub unsafe fn read_u32(&mut self) -> u32 {
```

`qsl/src/io.rs:129`

```rust
pub unsafe fn read_u64(&mut self) -> u64 {
```

`qsl/src/io.rs:137`

```rust
pub unsafe fn take(&mut self, len: usize) -> *const u8 {
```

`qsl/src/io.rs:147`

```rust
pub struct QslFastBitsReader {
```

`qsl/src/io.rs:158`

```rust
pub unsafe fn read_inline_string_into_with<const MAX: usize, H: crate::StringHeader>( &mut self, out: *mut crate::InlineString<MAX>, ) {
```

`qsl/src/io.rs:169`

```rust
pub unsafe fn read_raw_inline_string_into_with<const MAX: usize, H: crate::StringHeader>( &mut self, out: *mut crate::InlineString<MAX>, ) {
```

`qsl/src/io.rs:180`

```rust
pub unsafe fn read_inline_string_with<const MAX: usize, H: crate::StringHeader>( &mut self, ) -> crate::InlineString<MAX> {
```

`qsl/src/io.rs:191`

```rust
pub unsafe fn read_raw_inline_string_with<const MAX: usize, H: crate::StringHeader>( &mut self, ) -> crate::InlineString<MAX> {
```

`qsl/src/io.rs:202`

```rust
pub fn as_ptr(&self) -> *const u8 {
```

`qsl/src/io.rs:210`

```rust
pub unsafe fn read_raw_inline_string_into<const MAX: usize>( &mut self, output: *mut crate::InlineString<MAX>, ) {
```

`qsl/src/io.rs:222`

```rust
pub unsafe fn read_inline_string_into<const MAX: usize>( &mut self, output: *mut crate::InlineString<MAX>, ) {
```

`qsl/src/io.rs:234`

```rust
pub unsafe fn read_inline_string<const MAX: usize>(&mut self) -> crate::InlineString<MAX> {
```

`qsl/src/io.rs:245`

```rust
pub unsafe fn new(ptr: *const u8) -> Self {
```

`qsl/src/io.rs:256`

```rust
pub fn from(input: &[u8]) -> Self {
```

`qsl/src/io.rs:270`

```rust
pub unsafe fn read_bits(&mut self, width: u32) -> u64 {
```

`qsl/src/io.rs:289`

```rust
pub unsafe fn align(&mut self) {
```

`qsl/src/io.rs:298`

```rust
pub unsafe fn read_exact(&mut self, len: usize) -> *const u8 {
```

`qsl/src/io.rs:310`

```rust
pub unsafe fn read_u8(&mut self) -> u8 {
```

`qsl/src/io.rs:321`

```rust
pub unsafe fn read_u16(&mut self) -> u16 {
```

`qsl/src/io.rs:332`

```rust
pub unsafe fn read_u32(&mut self) -> u32 {
```

`qsl/src/io.rs:343`

```rust
pub unsafe fn read_u64(&mut self) -> u64 {
```

`qsl/src/io.rs:353`

```rust
pub unsafe fn read_i8(&mut self) -> i8 {
```

`qsl/src/io.rs:360`

```rust
pub unsafe fn read_i16(&mut self) -> i16 {
```

`qsl/src/io.rs:367`

```rust
pub unsafe fn read_i32(&mut self) -> i32 {
```

`qsl/src/io.rs:374`

```rust
pub unsafe fn read_i64(&mut self) -> i64 {
```

`qsl/src/io.rs:386`

```rust
pub struct QslUncheckedWriter {
```

`qsl/src/io.rs:396`

```rust
pub unsafe fn write_inline_string_with<const MAX: usize, H: crate::StringHeader>( &mut self, value: &crate::InlineString<MAX>, ) {
```

`qsl/src/io.rs:407`

```rust
pub fn as_mut_ptr(&self) -> *mut u8 {
```

`qsl/src/io.rs:414`

```rust
pub unsafe fn write_inline_string<const MAX: usize>( &mut self, value: &crate::InlineString<MAX>, ) {
```

`qsl/src/io.rs:425`

```rust
pub unsafe fn from_raw(ptr: *mut u8) -> Self {
```

`qsl/src/io.rs:436`

```rust
pub fn from(out: &mut [u8]) -> Self {
```

`qsl/src/io.rs:448`

```rust
pub unsafe fn write_bytes_unchecked(&mut self, bytes: &[u8]) {
```

`qsl/src/io.rs:458`

```rust
pub unsafe fn write_ptr_unchecked(&mut self, source: *const u8, len: usize) {
```

`qsl/src/io.rs:468`

```rust
pub unsafe fn write_zeros_unchecked(&mut self, count: usize) {
```

`qsl/src/io.rs:478`

```rust
pub unsafe fn write_bits_unchecked(&mut self, value: u64, width: u32) {
```

`qsl/src/io.rs:495`

```rust
pub unsafe fn flush_unchecked(&mut self) {
```

`qsl/src/io.rs:510`

```rust
pub unsafe fn claim_bytes(&mut self, n: usize) -> *mut u8 {
```

`qsl/src/io.rs:521`

```rust
pub unsafe fn write_bytes(&mut self, bytes: &[u8]) {
```

`qsl/src/io.rs:529`

```rust
pub unsafe fn write_zeros(&mut self, n: usize) {
```

`qsl/src/io.rs:537`

```rust
pub unsafe fn write_bits(&mut self, v: u64, width: u32) {
```

`qsl/src/io.rs:546`

```rust
pub unsafe fn flush(&mut self) {
```

`qsl/src/io.rs:553`

```rust
pub unsafe fn write_u8(&mut self, value: u8) {
```

`qsl/src/io.rs:563`

```rust
pub unsafe fn write_u16(&mut self, value: u16) {
```

`qsl/src/io.rs:573`

```rust
pub unsafe fn write_u32(&mut self, value: u32) {
```

`qsl/src/io.rs:583`

```rust
pub unsafe fn write_u64(&mut self, value: u64) {
```

`qsl/src/io.rs:592`

```rust
pub unsafe fn write_i8(&mut self, v: i8) {
```

`qsl/src/io.rs:599`

```rust
pub unsafe fn write_i16(&mut self, v: i16) {
```

`qsl/src/io.rs:606`

```rust
pub unsafe fn write_i32(&mut self, v: i32) {
```

`qsl/src/io.rs:613`

```rust
pub unsafe fn write_i64(&mut self, v: i64) {
```

`qsl/src/io.rs:620`

```rust
pub type QslWriter = QslRawWriter;
```

`qsl/src/io.rs:621`

```rust
pub type QslReader = QslRawReader;
```

`qsl/src/io.rs:622`

```rust
pub type QslBitsWriter = QslUncheckedWriter;
```

`qsl/src/io.rs:623`

```rust
pub type QslBitsReader = QslFastBitsReader;
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

`qsl/src/io_checked.rs:30`

```rust
pub fn from_raw(out: &'a mut crate::serialize::SerOut) -> Self {
```

`qsl/src/io_checked.rs:33`

```rust
pub fn position(&self) -> usize {
```

`qsl/src/io_checked.rs:36`

```rust
pub fn write(&mut self, bytes: &[u8]) -> QslResult<()> {
```

`qsl/src/io_checked.rs:52`

```rust
pub fn write_zeros(&mut self, len: usize) -> QslResult<()> {
```

`qsl/src/io_checked.rs:65`

```rust
pub fn write_u8(&mut self, v: u8) -> QslResult<()> {
```

`qsl/src/io_checked.rs:68`

```rust
pub fn write_u16(&mut self, v: u16) -> QslResult<()> {
```

`qsl/src/io_checked.rs:71`

```rust
pub fn write_u32(&mut self, v: u32) -> QslResult<()> {
```

`qsl/src/io_checked.rs:74`

```rust
pub fn write_u64(&mut self, v: u64) -> QslResult<()> {
```

`qsl/src/io_checked.rs:77`

```rust
pub fn write_i8(&mut self, v: i8) -> QslResult<()> {
```

`qsl/src/io_checked.rs:80`

```rust
pub fn write_i16(&mut self, v: i16) -> QslResult<()> {
```

`qsl/src/io_checked.rs:83`

```rust
pub fn write_i32(&mut self, v: i32) -> QslResult<()> {
```

`qsl/src/io_checked.rs:86`

```rust
pub fn write_i64(&mut self, v: i64) -> QslResult<()> {
```

`qsl/src/io_checked.rs:91`

```rust
pub struct QslCheckedReader<'a> {
```

`qsl/src/io_checked.rs:98`

```rust
pub fn new(bytes: &'a [u8]) -> Self {
```

`qsl/src/io_checked.rs:103`

```rust
pub unsafe fn from_raw_parts(ptr: *const u8, len: usize) -> Self {
```

`qsl/src/io_checked.rs:111`

```rust
pub fn position(&self) -> usize {
```

`qsl/src/io_checked.rs:114`

```rust
pub fn read_bytes(&mut self, n: usize) -> QslResult<&'a [u8]> {
```

`qsl/src/io_checked.rs:123`

```rust
pub fn read_u8(&mut self) -> QslResult<u8> {
```

`qsl/src/io_checked.rs:126`

```rust
pub fn read_u16(&mut self) -> QslResult<u16> {
```

`qsl/src/io_checked.rs:129`

```rust
pub fn read_u32(&mut self) -> QslResult<u32> {
```

`qsl/src/io_checked.rs:132`

```rust
pub fn read_u64(&mut self) -> QslResult<u64> {
```

`qsl/src/io_checked.rs:135`

```rust
pub fn read_i8(&mut self) -> QslResult<i8> {
```

`qsl/src/io_checked.rs:138`

```rust
pub fn read_i16(&mut self) -> QslResult<i16> {
```

`qsl/src/io_checked.rs:141`

```rust
pub fn read_i32(&mut self) -> QslResult<i32> {
```

`qsl/src/io_checked.rs:144`

```rust
pub fn read_i64(&mut self) -> QslResult<i64> {
```

`qsl/src/io_checked.rs:151`

```rust
pub struct QslCheckedBitsWriter<'a> {
```

`qsl/src/io_checked.rs:160`

```rust
pub fn from(out: &'a mut [u8]) -> Self {
```

`qsl/src/io_checked.rs:165`

```rust
pub unsafe fn from_raw_parts(ptr: *mut u8, len: usize) -> Self {
```

`qsl/src/io_checked.rs:175`

```rust
pub fn from_raw(out: &'a mut crate::serialize::SerOut) -> Self {
```

`qsl/src/io_checked.rs:178`

```rust
pub fn position(&self) -> usize {
```

`qsl/src/io_checked.rs:191`

```rust
pub fn write_bits(&mut self, mut value: u64, mut width: u32) -> QslResult<()> {
```

`qsl/src/io_checked.rs:213`

```rust
pub fn flush(&mut self) -> QslResult<()> {
```

`qsl/src/io_checked.rs:221`

```rust
pub fn write_bytes(&mut self, bytes: &[u8]) -> QslResult<()> {
```

`qsl/src/io_checked.rs:238`

```rust
pub fn write_zeros(&mut self, n: usize) -> QslResult<()> {
```

`qsl/src/io_checked.rs:252`

```rust
pub fn write_u8(&mut self, v: u8) -> QslResult<()> {
```

`qsl/src/io_checked.rs:255`

```rust
pub fn write_u16(&mut self, v: u16) -> QslResult<()> {
```

`qsl/src/io_checked.rs:258`

```rust
pub fn write_u32(&mut self, v: u32) -> QslResult<()> {
```

`qsl/src/io_checked.rs:261`

```rust
pub fn write_u64(&mut self, v: u64) -> QslResult<()> {
```

`qsl/src/io_checked.rs:264`

```rust
pub fn write_i8(&mut self, v: i8) -> QslResult<()> {
```

`qsl/src/io_checked.rs:267`

```rust
pub fn write_i16(&mut self, v: i16) -> QslResult<()> {
```

`qsl/src/io_checked.rs:270`

```rust
pub fn write_i32(&mut self, v: i32) -> QslResult<()> {
```

`qsl/src/io_checked.rs:273`

```rust
pub fn write_i64(&mut self, v: i64) -> QslResult<()> {
```

`qsl/src/io_checked.rs:278`

```rust
pub struct QslCheckedBitsReader<'a> {
```

`qsl/src/io_checked.rs:287`

```rust
pub fn new(input: &'a [u8]) -> Self {
```

`qsl/src/io_checked.rs:292`

```rust
pub unsafe fn from_raw_parts(ptr: *const u8, len: usize) -> Self {
```

`qsl/src/io_checked.rs:302`

```rust
pub fn position(&self) -> usize {
```

`qsl/src/io_checked.rs:305`

```rust
pub fn read_bits(&mut self, width: u32) -> QslResult<u64> {
```

`qsl/src/io_checked.rs:330`

```rust
pub fn align(&mut self) -> QslResult<()> {
```

`qsl/src/io_checked.rs:338`

```rust
pub fn read_bytes(&mut self, n: usize) -> QslResult<&'a [u8]> {
```

`qsl/src/io_checked.rs:348`

```rust
pub fn read_u8(&mut self) -> QslResult<u8> {
```

`qsl/src/io_checked.rs:351`

```rust
pub fn read_u16(&mut self) -> QslResult<u16> {
```

`qsl/src/io_checked.rs:354`

```rust
pub fn read_u32(&mut self) -> QslResult<u32> {
```

`qsl/src/io_checked.rs:357`

```rust
pub fn read_u64(&mut self) -> QslResult<u64> {
```

`qsl/src/io_checked.rs:360`

```rust
pub fn read_i8(&mut self) -> QslResult<i8> {
```

`qsl/src/io_checked.rs:363`

```rust
pub fn read_i16(&mut self) -> QslResult<i16> {
```

`qsl/src/io_checked.rs:366`

```rust
pub fn read_i32(&mut self) -> QslResult<i32> {
```

`qsl/src/io_checked.rs:369`

```rust
pub fn read_i64(&mut self) -> QslResult<i64> {
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

`qsl/src/modernization.rs:3`

```rust
pub use qsl_lexer::modernization::*;
```

`qsl/src/modernization.rs:13`

```rust
pub unsafe fn $name<const N: usize>(source: *const u8) -> $word {
```

`qsl/src/modernization.rs:33`

```rust
pub unsafe fn load_exact<const N: usize>(source: *const u8) -> u128 {
```

`qsl/src/modernization.rs:39`

```rust
pub struct InlineArray<T, const N: usize> {
```

`qsl/src/modernization.rs:49`

```rust
pub const fn new() -> Self {
```

`qsl/src/modernization.rs:56`

```rust
pub fn len(&self) -> usize {
```

`qsl/src/modernization.rs:59`

```rust
pub fn is_empty(&self) -> bool {
```

`qsl/src/modernization.rs:62`

```rust
pub fn capacity(&self) -> usize {
```

`qsl/src/modernization.rs:65`

```rust
pub fn as_slice(&self) -> &[T] {
```

`qsl/src/modernization.rs:68`

```rust
pub fn as_mut_slice(&mut self) -> &mut [T] {
```

`qsl/src/modernization.rs:73`

```rust
pub fn push(&mut self, value: T) -> QslResult<()> {
```

`qsl/src/modernization.rs:81`

```rust
pub fn into_array(mut self) -> QslResult<[T;
```

`qsl/src/modernization.rs:89`

```rust
pub fn clear(&mut self) {
```

`qsl/src/modernization.rs:98`

```rust
pub unsafe fn copy_native_into(source: *const u8, len: usize, out: *mut Self) where T: Copy, {
```

`qsl/src/modernization.rs:109`

```rust
pub fn try_from_slice(values: &[T]) -> QslResult<Self> where T: Copy, {
```

`qsl/src/modernization.rs:127`

```rust
pub fn try_from_iter(values: impl IntoIterator<Item = T>) -> QslResult<Self> {
```

`qsl/src/modernization.rs:162`

```rust
pub struct StackString<const N: usize>(InlineArray<u8, N>);
```

`qsl/src/modernization.rs:164`

```rust
pub fn new() -> Self {
```

`qsl/src/modernization.rs:167`

```rust
pub fn try_from_str(text: &str) -> QslResult<Self> {
```

`qsl/src/modernization.rs:173`

```rust
pub unsafe fn copy_into(source: *const u8, len: usize, out: *mut Self) {
```

`qsl/src/modernization.rs:176`

```rust
pub fn as_str(&self) -> &str {
```

`qsl/src/modernization.rs:179`

```rust
pub fn as_bytes(&self) -> &[u8] {
```

`qsl/src/modernization.rs:182`

```rust
pub fn len(&self) -> usize {
```

`qsl/src/modernization.rs:185`

```rust
pub fn is_empty(&self) -> bool {
```

`qsl/src/modernization.rs:328`

```rust
pub const fn new(value: u32) -> Self {
```

`qsl/src/modernization.rs:335`

```rust
pub fn try_take(&self) -> bool {
```

`qsl/src/modernization.rs:344`

```rust
pub fn wait(&self, expected: u32) {
```

`qsl/src/modernization.rs:354`

```rust
pub fn wake_all(&self) {
```

`qsl/src/modernization.rs:414`

```rust
pub unsafe fn load_word(source: *const u8, count: usize) -> u128 {
```

`qsl/src/modernization.rs:420`

```rust
pub unsafe fn store_word(destination: *mut u8, value: u128, count: usize) {
```

`qsl/src/modernization.rs:425`

```rust
pub struct CachedFrame<const N: usize> {
```

`qsl/src/modernization.rs:435`

```rust
pub const fn new() -> Self {
```

`qsl/src/modernization.rs:441`

```rust
pub fn as_slice(&self) -> &[u8] {
```

`qsl/src/modernization.rs:444`

```rust
pub fn as_mut_ptr(&mut self) -> *mut u8 {
```

`qsl/src/modernization.rs:449`

```rust
pub unsafe fn set_len(&mut self, len: usize) {
```

`qsl/src/modernization.rs:456`

```rust
pub struct NativeLeaf {
```

`qsl/src/modernization.rs:463`

```rust
pub struct NativeLayout {
```

`qsl/src/modernization.rs:468`

```rust
pub const EMPTY: Self = Self {
```

`qsl/src/modernization.rs:477`

```rust
pub const fn leaf(bytes: usize, kind: u16, elements: usize) -> Self {
```

`qsl/src/modernization.rs:488`

```rust
pub const fn fields(fields: &[(usize, Self)]) -> Self {
```

`qsl/src/modernization.rs:505`

```rust
pub const fn compatible(&self, other: &Self) -> bool {
```

`qsl/src/modernization.rs:518`

```rust
pub const fn same_fields(&self, other: &Self) -> bool {
```

`qsl/src/modernization.rs:541`

```rust
pub unsafe trait NativeField: Copy {
```

`qsl/src/modernization.rs:543`

```rust
fn validate_native(bytes: &[u8]) -> QslResult<()> {
```

`qsl/src/modernization.rs:551`

```rust
unsafe fn validate_native_fields(address: *const u8) -> QslResult<()>;
```

`qsl/src/modernization.rs:590`

```rust
pub static ZERO_WORD: [u8;
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

## qsl/src/payload_quant.rs

`qsl/src/payload_quant.rs:4`

```rust
pub use qsl_lexer::payload_quant::{PayloadQuantPlan, QuantField};
```

`qsl/src/payload_quant.rs:12`

```rust
pub fn selected_address<T>(mut value: *const T) -> *const T {
```

`qsl/src/payload_quant.rs:27`

```rust
pub fn scalar_operand(mut value: u64) -> u64 {
```

`qsl/src/payload_quant.rs:42`

```rust
pub unsafe trait QuantizedRecord: Serializable {
```

`qsl/src/payload_quant.rs:47`

```rust
fn quant_validate_values(&self) -> QslResult<()>;
```

`qsl/src/payload_quant.rs:51`

```rust
unsafe fn quant_write_prepared(&self, out: *mut u8) -> usize;
```

`qsl/src/payload_quant.rs:55`

```rust
unsafe fn quant_read_padded(bytes: *const u8, len: usize, out: *mut Self);
```

`qsl/src/payload_quant.rs:57`

```rust
pub trait CheckedQuantizedRecord: QuantizedRecord {
```

`qsl/src/payload_quant.rs:58`

```rust
fn quant_validate(input: &[u8]) -> QslResult<()>;
```

`qsl/src/payload_quant.rs:61`

```rust
pub struct QuantizedFrame<'a, T: QuantizedRecord> {
```

`qsl/src/payload_quant.rs:67`

```rust
pub fn try_new(storage: &'a [u8], len: usize) -> QslResult<Self> {
```

`qsl/src/payload_quant.rs:84`

```rust
pub unsafe fn from_validated(storage: &[u8], len: usize) -> QuantizedFrame<'_, T> {
```

`qsl/src/payload_quant.rs:91`

```rust
pub fn as_slice(&self) -> &[u8] {
```

`qsl/src/payload_quant.rs:94`

```rust
pub fn decode_into<'out>(&self, out: &'out mut std::mem::MaybeUninit<T>) -> &'out mut T {
```

`qsl/src/payload_quant.rs:100`

```rust
pub fn decode_boxed(&self) -> Box<T> {
```

`qsl/src/payload_quant.rs:109`

```rust
pub struct QuantizedOut<T: QuantizedRecord> {
```

`qsl/src/payload_quant.rs:115`

```rust
pub fn encode(value: &T) -> QslResult<Self> {
```

`qsl/src/payload_quant.rs:131`

```rust
pub fn as_slice(&self) -> &[u8] {
```

`qsl/src/payload_quant.rs:134`

```rust
pub fn frame(&self) -> QuantizedFrame<'_, T> {
```

`qsl/src/payload_quant.rs:154`

```rust
pub struct QuantizedRegions<T: QuantizedRecord, const CAP: usize, const N: usize> {
```

`qsl/src/payload_quant.rs:164`

```rust
pub const fn new() -> Self {
```

`qsl/src/payload_quant.rs:170`

```rust
pub fn encode(&'static self, value: &T) -> QslResult<QuantizedLease<T, CAP, N>> {
```

`qsl/src/payload_quant.rs:192`

```rust
pub struct QuantizedLease<T: QuantizedRecord, const CAP: usize, const N: usize> {
```

`qsl/src/payload_quant.rs:198`

```rust
pub fn as_slice(&self) -> &[u8] {
```

`qsl/src/payload_quant.rs:201`

```rust
pub fn frame(&self) -> QuantizedFrame<'_, T> {
```

`qsl/src/payload_quant.rs:225`

```rust
pub trait QuantizedDestination<T: QuantizedRecord> {
```

`qsl/src/payload_quant.rs:229`

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

`qsl/src/rt.rs:26`

```rust
pub const fn new() -> Self {
```

`qsl/src/rt.rs:34`

```rust
pub fn try_acquire(&'static self) -> QslResult<RegionLease<CAP, N>> {
```

`qsl/src/rt.rs:65`

```rust
pub fn acquire(&'static self) -> RegionLease<CAP, N> {
```

`qsl/src/rt.rs:74`

```rust
pub const fn new_zeroed() -> Self {
```

`qsl/src/rt.rs:83`

```rust
pub struct RegionLease<const CAP: usize, const N: usize> {
```

`qsl/src/rt.rs:89`

```rust
pub fn capacity(&self) -> usize {
```

`qsl/src/rt.rs:92`

```rust
pub fn as_slice(&self) -> &[u8] {
```

`qsl/src/rt.rs:101`

```rust
pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
```

`qsl/src/rt.rs:106`

```rust
pub unsafe fn set_len(&mut self, len: usize) {
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

`qsl/src/serialize.rs:22`

```rust
unsafe fn serialize(&self) -> Vec<u8> {
```

`qsl/src/serialize.rs:35`

```rust
unsafe fn serialize_unsafe(&self, to: *mut u8);
```

`qsl/src/serialize.rs:41`

```rust
unsafe fn serialize_into_unsafe(&self, to: *mut u8) -> usize {
```

`qsl/src/serialize.rs:49`

```rust
unsafe fn serialize_to(&self, to: &mut [u8]) -> usize {
```

`qsl/src/serialize.rs:56`

```rust
unsafe fn deserialize(input: &[u8]) -> Self {
```

`qsl/src/serialize.rs:65`

```rust
unsafe fn deserialize_into(bytes: *const u8, len: usize, out: *mut Self) {
```

`qsl/src/serialize.rs:75`

```rust
unsafe fn deserialize_reuse(input: &[u8], output: &mut Self) {
```

`qsl/src/serialize.rs:83`

```rust
unsafe fn deserialize_prepared_into(bytes: *const u8, len: usize, output: &mut Self) {
```

`qsl/src/serialize.rs:91`

```rust
unsafe fn deserialize_unsafe(bytes: *const u8) -> Self;
```

`qsl/src/serialize.rs:96`

```rust
unsafe fn deserialize_from_raw_parts(bytes: *const u8, _len: usize) -> Self {
```

`qsl/src/serialize.rs:103`

```rust
pub trait BorrowSerializable<'a>: Sized {
```

`qsl/src/serialize.rs:109`

```rust
fn get_size() -> usize;
```

`qsl/src/serialize.rs:111`

```rust
fn serialized_size(&self) -> usize;
```

`qsl/src/serialize.rs:117`

```rust
unsafe fn serialize(&self) -> Vec<u8> {
```

`qsl/src/serialize.rs:129`

```rust
unsafe fn serialize_unsafe(&self, to: *mut u8);
```

`qsl/src/serialize.rs:135`

```rust
unsafe fn serialize_into_unsafe(&self, to: *mut u8) -> usize {
```

`qsl/src/serialize.rs:142`

```rust
unsafe fn serialize_to(&self, to: &mut [u8]) -> usize {
```

`qsl/src/serialize.rs:148`

```rust
unsafe fn deserialize_borrowed(input: &'a [u8]) -> Self {
```

`qsl/src/serialize.rs:157`

```rust
unsafe fn deserialize_borrowed_into(bytes: *const u8, len: usize, out: *mut Self) {
```

`qsl/src/serialize.rs:164`

```rust
unsafe fn deserialize_borrowed_unsafe(bytes: *const u8) -> Self;
```

`qsl/src/serialize.rs:169`

```rust
unsafe fn deserialize_borrowed_from_raw_parts(bytes: *const u8, _len: usize) -> Self {
```

`qsl/src/serialize.rs:174`

```rust
pub struct SerOut {
```

`qsl/src/serialize.rs:182`

```rust
pub fn new(len: usize) -> Self {
```

`qsl/src/serialize.rs:196`

```rust
pub fn with_capacity_aligned(capacity: usize, alignment: usize) -> Self {
```

`qsl/src/serialize.rs:213`

```rust
pub fn capacity(&self) -> usize {
```

`qsl/src/serialize.rs:217`

```rust
pub fn alignment(&self) -> usize {
```

`qsl/src/serialize.rs:221`

```rust
pub fn clear(&mut self) {
```

`qsl/src/serialize.rs:228`

```rust
pub unsafe fn set_len(&mut self, len: usize) {
```

`qsl/src/serialize.rs:236`

```rust
pub unsafe fn from_raw(ptr: *mut u8, layout: std::alloc::Layout, len: usize) -> Self {
```

`qsl/src/serialize.rs:246`

```rust
pub fn as_mut_ptr(&mut self) -> *mut u8 {
```

`qsl/src/serialize.rs:291`

```rust
pub fn len(&self) -> usize {
```

`qsl/src/serialize.rs:295`

```rust
pub fn is_empty(&self) -> bool {
```

`qsl/src/serialize.rs:300`

```rust
pub fn copy_from_slice(&mut self, bytes: &[u8]) -> crate::QslResult<()> {
```

`qsl/src/serialize.rs:314`

```rust
pub unsafe fn copy_from_raw_parts(&mut self, source: *const u8, len: usize) {
```

`qsl/src/serialize.rs:319`

```rust
pub fn spare_capacity_mut(&mut self) -> &mut [std::mem::MaybeUninit<u8>] {
```

`qsl/src/serialize.rs:329`

```rust
pub fn into_utf8_guard(self) -> Result<crate::Utf8CopyGuard, (std::str::Utf8Error, Self)> {
```

`qsl/src/serialize.rs:335`

```rust
pub trait Sealed {
```

`qsl/src/serialize.rs:342`

```rust
pub trait DeserializeDestination<T: Serializable>: destination_private::Sealed {
```

`qsl/src/serialize.rs:347`

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

## qsl/src/strings.rs

`qsl/src/strings.rs:11`

```rust
pub use qsl_lexer::wire::{STRING_COMPRESSION_MIN_BYTES, STRING_ENCODER_REVISION};
```

`qsl/src/strings.rs:16`

```rust
pub trait Sealed {
```

`qsl/src/strings.rs:24`

```rust
pub trait StringHeader: header_sealed::Sealed {
```

`qsl/src/strings.rs:29`

```rust
unsafe fn load(input: *const u8) -> u32;
```

`qsl/src/strings.rs:32`

```rust
unsafe fn store(output: *mut u8, value: u32);
```

`qsl/src/strings.rs:60`

```rust
pub struct InlineString<const MAX: usize> {
```

`qsl/src/strings.rs:67`

```rust
pub const MAX_BYTES: usize = MAX;
```

`qsl/src/strings.rs:69`

```rust
pub const HEADER_BYTES: usize = qsl_lexer::wire::inline_string_header_bytes(MAX);
```

`qsl/src/strings.rs:71`

```rust
pub const OFFSET_BYTES: usize = if MAX <= 255 {
```

`qsl/src/strings.rs:76`

```rust
pub const fn new() -> Self {
```

`qsl/src/strings.rs:88`

```rust
pub unsafe fn from_str_unchecked(value: &str) -> Self {
```

`qsl/src/strings.rs:102`

```rust
pub const fn spaces<const LEN: usize>() -> Self {
```

`qsl/src/strings.rs:116`

```rust
pub fn len(&self) -> usize {
```

`qsl/src/strings.rs:122`

```rust
pub fn is_empty(&self) -> bool {
```

`qsl/src/strings.rs:128`

```rust
pub fn as_str(&self) -> &str {
```

`qsl/src/strings.rs:140`

```rust
pub fn wire_capacity(&self) -> usize {
```

`qsl/src/strings.rs:149`

```rust
pub unsafe fn encode_to(&self, output: *mut u8) -> usize {
```

`qsl/src/strings.rs:157`

```rust
pub unsafe fn encode_to_with<H: StringHeader>(&self, output: *mut u8) -> usize {
```

`qsl/src/strings.rs:165`

```rust
pub unsafe fn encode_raw_to(&self, output: *mut u8) -> usize {
```

`qsl/src/strings.rs:180`

```rust
pub unsafe fn decode_into(input: *const u8, output: *mut Self) -> usize {
```

`qsl/src/strings.rs:190`

```rust
pub unsafe fn decode_into_with<H: StringHeader>(input: *const u8, output: *mut Self) -> usize {
```

`qsl/src/strings.rs:201`

```rust
pub unsafe fn decode_raw_into_with<H: StringHeader>( input: *const u8, output: *mut Self, ) -> usize {
```

`qsl/src/strings.rs:222`

```rust
pub unsafe fn decode_from_with<H: StringHeader>(input: *const u8) -> (Self, usize) {
```

`qsl/src/strings.rs:232`

```rust
pub unsafe fn decode_raw_from_with<H: StringHeader>(input: *const u8) -> (Self, usize) {
```

`qsl/src/strings.rs:242`

```rust
pub unsafe fn decode_raw_into(input: *const u8, output: *mut Self) -> usize {
```

`qsl/src/strings.rs:259`

```rust
pub unsafe fn decode_from(input: *const u8) -> (Self, usize) {
```

`qsl/src/strings.rs:411`

```rust
pub unsafe fn encode_bounded<const MAX: usize>( input: *const u8, len: usize, output: *mut u8, ) -> usize {
```

`qsl/src/strings.rs:429`

```rust
pub unsafe fn encode_bounded_with<const MAX: usize, H: StringHeader>( input: *const u8, len: usize, output: *mut u8, ) -> usize {
```

`qsl/src/strings.rs:751`

```rust
pub unsafe fn decode_bounded<const MAX: usize>( input: *const u8, output: *mut u8, ) -> (usize, usize) {
```

`qsl/src/strings.rs:768`

```rust
pub unsafe fn decode_bounded_with<const MAX: usize, H: StringHeader>( input: *const u8, output: *mut u8, ) -> (usize, usize) {
```

`qsl/src/strings.rs:856`

```rust
pub const FIXED_UTF8_CODEC_VERSION: u8 = 2;
```

`qsl/src/strings.rs:864`

```rust
pub unsafe fn encode_fixed_utf8<const MAX: usize>(input: *const u8, output: *mut u8) -> usize {
```

`qsl/src/strings.rs:886`

```rust
pub unsafe fn decode_fixed_utf8<const MAX: usize>(input: *const u8, output: *mut u8) -> usize {
```

`qsl/src/strings.rs:908`

```rust
pub struct Utf8Guard<'a> {
```

`qsl/src/strings.rs:913`

```rust
pub const fn from_str(value: &'a str) -> Self {
```

`qsl/src/strings.rs:919`

```rust
pub fn try_from_bytes(bytes: &'a [u8]) -> Result<Self, str::Utf8Error> {
```

`qsl/src/strings.rs:930`

```rust
pub const unsafe fn from_bytes_unchecked(bytes: &'a [u8]) -> Self {
```

`qsl/src/strings.rs:934`

```rust
pub const fn as_bytes(self) -> &'a [u8] {
```

`qsl/src/strings.rs:938`

```rust
pub const fn as_str(self) -> &'a str {
```

`qsl/src/strings.rs:1316`

```rust
pub type Utf8View<'a> = Utf8Guard<'a>;
```

`qsl/src/strings.rs:1323`

```rust
pub struct Utf8CopyGuard {
```

`qsl/src/strings.rs:1327`

```rust
pub fn with_capacity(capacity: usize) -> Self {
```

`qsl/src/strings.rs:1332`

```rust
pub fn copy_of(value: &str) -> Self {
```

`qsl/src/strings.rs:1339`

```rust
pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, str::Utf8Error> {
```

`qsl/src/strings.rs:1346`

```rust
pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> Self {
```

`qsl/src/strings.rs:1352`

```rust
pub fn capacity(&self) -> usize {
```

`qsl/src/strings.rs:1356`

```rust
pub fn len(&self) -> usize {
```

`qsl/src/strings.rs:1360`

```rust
pub fn is_empty(&self) -> bool {
```

`qsl/src/strings.rs:1364`

```rust
pub fn as_bytes(&self) -> &[u8] {
```

`qsl/src/strings.rs:1368`

```rust
pub fn as_str(&self) -> &str {
```

`qsl/src/strings.rs:1371`

```rust
pub fn clear(&mut self) {
```

`qsl/src/strings.rs:1376`

```rust
pub fn copy_from_str(&mut self, value: &str) -> crate::QslResult<()> {
```

`qsl/src/strings.rs:1380`

```rust
pub fn try_copy_from_bytes(&mut self, bytes: &[u8]) -> crate::QslResult<()> {
```

`qsl/src/strings.rs:1394`

```rust
pub unsafe fn copy_from_raw_parts_unchecked(&mut self, source: *const u8, len: usize) {
```

`qsl/src/strings.rs:1402`

```rust
pub unsafe fn decode_bounded_into<const MAX: usize>(&mut self, input: *const u8) -> usize {
```

`qsl/src/strings.rs:1408`

```rust
pub fn try_from_ser_out( storage: crate::SerOut, ) -> Result<Self, (str::Utf8Error, crate::SerOut)> {
```

`qsl/src/strings.rs:1418`

```rust
pub unsafe fn from_ser_out_unchecked(storage: crate::SerOut) -> Self {
```

`qsl/src/strings.rs:1422`

```rust
pub fn into_ser_out(self) -> crate::SerOut {
```

## qsl/src/validation.rs

`qsl/src/validation.rs:6`

```rust
pub use qsl_lexer::config::DEFAULT_MAX_DATA_SIZE;
```

`qsl/src/validation.rs:8`

```rust
pub fn check_size(size: usize, maximum: usize) -> QslResult<()> {
```

`qsl/src/validation.rs:17`

```rust
pub struct ValueBudget {
```

`qsl/src/validation.rs:21`

```rust
pub fn new(maximum: usize) -> Self {
```

`qsl/src/validation.rs:24`

```rust
pub fn charge(&mut self, bytes: usize) -> QslResult<()> {
```

`qsl/src/validation.rs:94`

```rust
pub struct ValidationCursor<'a> {
```

`qsl/src/validation.rs:100`

```rust
pub fn new(input: &'a [u8]) -> Self {
```

`qsl/src/validation.rs:108`

```rust
pub fn with_limit(input: &'a [u8], maximum: usize) -> QslResult<Self> {
```

`qsl/src/validation.rs:111`

```rust
pub fn with_budget( input: &'a [u8], maximum_frame: usize, decoded_budget: usize, ) -> QslResult<Self> {
```

`qsl/src/validation.rs:123`

```rust
pub fn charge(&mut self, bytes: usize) -> QslResult<()> {
```

`qsl/src/validation.rs:126`

```rust
pub fn check_values(&self, bytes: usize, bits: usize) -> QslResult<()> {
```

`qsl/src/validation.rs:139`

```rust
pub fn check_allocation(&mut self, bytes: usize) -> QslResult<()> {
```

`qsl/src/validation.rs:149`

```rust
pub fn text(&mut self, n: usize, min: usize, max: usize) -> QslResult<&'a str> {
```

`qsl/src/validation.rs:156`

```rust
pub fn payload(&mut self, bytes: usize) -> QslResult<&'a [u8]> {
```

`qsl/src/validation.rs:160`

```rust
pub fn bits(&mut self, width: usize) -> QslResult<u64> {
```

`qsl/src/validation.rs:182`

```rust
pub fn align(&mut self) -> QslResult<()> {
```

`qsl/src/validation.rs:189`

```rust
pub fn position(&self) -> usize {
```

`qsl/src/validation.rs:192`

```rust
pub fn bytes(&mut self, n: usize) -> QslResult<&'a [u8]> {
```

`qsl/src/validation.rs:200`

```rust
pub fn uint(&mut self, n: usize) -> QslResult<u64> {
```

`qsl/src/validation.rs:208`

```rust
pub fn utf8(&mut self, n: usize, min: usize, max: usize) -> QslResult<()> {
```

`qsl/src/validation.rs:215`

```rust
pub fn finish(mut self) -> QslResult<()> {
```

`qsl/src/validation.rs:224`

```rust
pub fn compressed<const MAX: usize>( &mut self, min: usize, raw_only: bool, fixed: bool, ) -> QslResult<()> {
```

`qsl/src/validation.rs:235`

```rust
pub fn decode_compressed_owned<const MAX: usize>( &mut self, min: usize, raw_only: bool, fixed: bool, ) -> QslResult<String> {
```

`qsl/src/validation.rs:300`

```rust
pub fn decode_compressed<const MAX: usize>( &mut self, min: usize, raw_only: bool, fixed: bool, ) -> QslResult<crate::InlineString<MAX>> {
```
