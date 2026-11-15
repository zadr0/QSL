//! Shared build-time generation settings.

/// Default bound for input, restored frames and dynamic values.
pub const DEFAULT_MAX_DATA_SIZE: usize = 65535;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenerationConfig {
    pub use_hash: crate::SchemaHashAlgorithm,
    pub max_data_size: usize,
    pub qsl_rt: bool,
    pub qsl_rt_default: usize,
    pub stack_limit: usize,
    /// Build-time CPU cost profile; never changes canonical wire bytes.
    pub cpu_profile: crate::optimizer::registers::CpuProfile,
}
impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            use_hash: crate::SchemaHashAlgorithm::default(),
            max_data_size: DEFAULT_MAX_DATA_SIZE,
            qsl_rt: false,
            qsl_rt_default: 16,
            stack_limit: 256,
            cpu_profile: crate::optimizer::registers::CpuProfile::Portable,
        }
    }
}
