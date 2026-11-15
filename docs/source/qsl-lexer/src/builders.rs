//! Shared, target-independent declaration contracts.
use crate::{GenerationConfig, QslError, QslField, QslResult, QslType, SchemaHashAlgorithm};

/// DEQ settings shared by text input and the programmatic builder.
#[derive(Clone, Debug, Default)]
pub struct DeqBuilder {
    config: GenerationConfig,
}
impl DeqBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn from_config(config: GenerationConfig) -> Self {
        Self { config }
    }
    pub fn with_rt(&mut self) -> &mut Self {
        self.config.qsl_rt = true;
        self
    }
    pub fn default_rt_slots(&mut self, slots: u32) -> &mut Self {
        self.config.qsl_rt_default = slots as usize;
        self
    }
    /// Bound input, restored frames, and aggregate dynamic allocations, in bytes.
    pub fn max_data_size(&mut self, bytes: u32) -> &mut Self {
        self.config.max_data_size = bytes as usize;
        self
    }
    /// Independent upper bound for inline/stack storage, in bytes.
    pub fn stack_limit(&mut self, bytes: u32) -> &mut Self {
        self.config.stack_limit = bytes as usize;
        self
    }
    /// Select a build-time instruction cost profile. The consumer must still
    /// enable the matching target features; no CPU-dependent wire is introduced.
    pub fn cpu_profile(&mut self, profile: crate::optimizer::registers::CpuProfile) -> &mut Self {
        self.config.cpu_profile = profile;
        self
    }
    #[allow(non_snake_case)]
    pub fn use_xxHash128(&mut self) -> &mut Self {
        self.config.use_hash = SchemaHashAlgorithm::Xxh3_128;
        self
    }
    pub fn use_xxhash128(&mut self) -> &mut Self {
        self.use_xxHash128()
    }
    pub fn build(&self) -> QslResult<GenerationConfig> {
        if !(1..=64).contains(&self.config.qsl_rt_default) {
            return Err(QslError::InvalidSchema);
        }
        Ok(self.config)
    }
}

/// Exact schema type metadata. This does not promise native-layout compatibility.
pub trait QslFieldType {
    fn qsl_type() -> QslType;
    const OPTIONAL: bool = false;
    const VALID: bool = true;
}
macro_rules! integer { ($($t:ty => ($bits:expr,$signed:expr)),*) => { $(
    impl QslFieldType for $t { fn qsl_type()->QslType { QslType::Integer{bits:$bits,signed:$signed,min:0} } }
)* }; }
impl QslFieldType for u8 {
    fn qsl_type() -> QslType {
        QslType::Byte
    }
}
integer!(i8=>(8,true),u16=>(16,false),i16=>(16,true),u32=>(32,false),i32=>(32,true),u64=>(64,false),i64=>(64,true));
impl QslFieldType for u128 {
    fn qsl_type() -> QslType {
        QslType::DomainInteger(
            crate::domain::IntegerDomain::unsigned(128, 0, u128::MAX)
                .expect("full unsigned domain")
                .into(),
        )
    }
}
impl QslFieldType for i128 {
    fn qsl_type() -> QslType {
        QslType::DomainInteger(
            crate::domain::IntegerDomain::signed(128, i128::MIN, i128::MAX)
                .expect("full signed domain")
                .into(),
        )
    }
}
impl QslFieldType for String {
    fn qsl_type() -> QslType {
        QslType::String(0)
    }
}
impl<T: QslFieldType> QslFieldType for Vec<T> {
    const VALID: bool = T::VALID && !T::OPTIONAL;
    fn qsl_type() -> QslType {
        QslType::Array(Box::new(T::qsl_type()), 0)
    }
}
impl<T: QslFieldType, const N: usize> QslFieldType for [T; N] {
    const VALID: bool = T::VALID && !T::OPTIONAL;
    fn qsl_type() -> QslType {
        QslType::FixedArray(Box::new(T::qsl_type()), N)
    }
}
impl<T: QslFieldType> QslFieldType for Option<T> {
    fn qsl_type() -> QslType {
        T::qsl_type()
    }
    const OPTIONAL: bool = true;
    const VALID: bool = T::VALID && !T::OPTIONAL;
}
/// Build-time field expansion; nested macros return their expanded logical fields.
/// Implementations must not claim that Rust object bytes are wire bytes.
pub trait QslMacro {
    fn qsl_fields() -> QslResult<Vec<QslField>>;
}

impl QslFieldType for &str {
    fn qsl_type() -> QslType {
        QslType::BorrowedString { min: 0, max: None }
    }
}
impl<const N: usize, L: crate::storage::LengthWord> QslFieldType for crate::StackString<N, L> {
    const VALID: bool = N <= L::MAX_LEN;
    fn qsl_type() -> QslType {
        QslType::RangedString { min: 0, max: N }
    }
}
impl<const N: usize, L: crate::storage::LengthWord> QslFieldType for crate::InlineString<N, L> {
    const VALID: bool = N <= L::MAX_LEN && N <= u16::MAX as usize;
    fn qsl_type() -> QslType {
        QslType::CompressedString { min: 0, max: N }
    }
}

/// Box changes physical ownership, never the logical field codec.
impl<T: QslFieldType> QslFieldType for Box<T> {
    const VALID: bool = T::VALID && !T::OPTIONAL;
    fn qsl_type() -> QslType {
        T::qsl_type()
    }
}
impl<T: QslFieldType, const N: usize, L: crate::storage::LengthWord> QslFieldType
    for qsl_core::InlineArray<T, N, L>
{
    const VALID: bool = T::VALID && !T::OPTIONAL && N <= L::MAX_LEN;
    fn qsl_type() -> QslType {
        QslType::RangedArray {
            element: Box::new(T::qsl_type()),
            min: 0,
            max: N,
        }
    }
}
