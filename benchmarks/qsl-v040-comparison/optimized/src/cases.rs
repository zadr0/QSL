use crate::{fixed, fixture, zero_copy, Codec, Result, Verified, CODECS, MAX_FRAME};
use std::hint::black_box;

pub const COUNT: usize = 256;
pub const VALIDATION_OFFSET: usize = 1024;
#[derive(Clone, Copy)]
pub struct Case {
    pub workload: &'static str,
    pub operation: &'static str,
    pub profile: &'static str,
    pub trusted: bool,
}
impl Case {
    pub fn id(&self) -> String {
        let suffix = if self.trusted { "_trusted" } else { "" };
        format!(
            "{}/{}/{}{suffix}",
            self.workload, self.operation, self.profile
        )
    }
    pub fn new_input(
        self,
        offset: usize,
        index: usize,
        output: &mut [u8; MAX_FRAME],
    ) -> Result<Input> {
        let i = offset + index;
        Ok(match self.workload {
            "fixed" => Input::Fixed(fixed::Fixed::new(self.profile, fixed::values(i), output)?),
            "text_128" | "text_2048" => {
                let length = if self.workload == "text_128" {
                    128
                } else {
                    2048
                };
                Input::Borrowed(zero_copy::Borrowed::new(
                    self.profile,
                    (i + 1) as u32,
                    &zero_copy::text(i, length),
                    output,
                )?)
            }
            _ => {
                let codec: Codec = self.profile.parse()?;
                Input::Envelope(Verified::new(
                    codec,
                    &fixture::make(self.workload, i),
                    output,
                )?)
            }
        })
    }
    pub fn warm(self, offset: usize, output: &mut [u8; MAX_FRAME]) -> Result<Vec<Input>> {
        let records = (0..COUNT)
            .map(|i| self.new_input(offset, i, output))
            .collect::<Result<Vec<_>>>()?;
        for value in &records {
            black_box(value.wire_len());
            self.run(value, output)?;
        }
        Ok(records)
    }
    #[inline]
    pub fn run(self, record: &Input, output: &mut [u8; MAX_FRAME]) -> Result<()> {
        match (self.operation, record) {
            ("allocate", Input::Envelope(v)) => {
                black_box(v.allocate(self.trusted)?);
            }
            ("allocate", Input::Fixed(v)) => {
                black_box(v.allocate(self.trusted)?);
            }
            ("buffer", Input::Envelope(v)) => {
                let n = v.write(output, self.trusted)?;
                black_box(&output[..n]);
            }
            ("buffer", Input::Fixed(v)) => {
                let n = v.write(output, self.trusted)?;
                black_box(&output[..n]);
            }
            ("owned", Input::Envelope(v)) => {
                black_box(v.read(self.trusted)?);
            }
            ("owned", Input::Fixed(v)) => {
                black_box(v.read(self.trusted)?);
            }
            ("view", Input::Borrowed(v)) => {
                black_box(v.with(self.trusted, zero_copy::sample)?);
            }
            _ => return Err("invalid benchmark contract".into()),
        }
        Ok(())
    }
}
pub enum Input {
    Envelope(Verified),
    Fixed(fixed::Fixed),
    Borrowed(zero_copy::Borrowed),
}
impl Input {
    pub fn wire_len(&self) -> usize {
        match self {
            Self::Envelope(v) => v.frame().len(),
            Self::Fixed(v) => v.frame().len(),
            Self::Borrowed(v) => v.len(),
        }
    }
}
pub fn registry() -> Vec<Case> {
    let mut output = Vec::new();
    let mut register = |workload, operation, profile: &'static str| {
        output.push(Case {
            workload,
            operation,
            profile,
            trusted: false,
        });
        if profile.starts_with("qsl_") {
            output.push(Case {
                workload,
                operation,
                profile,
                trusted: true,
            });
        }
    };
    for workload in fixture::WORKLOADS.into_iter().chain(["fixed"]) {
        let profiles = if workload == "fixed" {
            fixed::PROFILES.to_vec()
        } else {
            CODECS.into_iter().map(Codec::name).collect()
        };
        for operation in ["allocate", "buffer", "owned"] {
            for &profile in &profiles {
                register(workload, operation, profile);
            }
        }
    }
    for workload in ["text_128", "text_2048"] {
        for profile in zero_copy::PROFILES {
            register(workload, "view", profile);
        }
    }
    output
}
pub fn selection() -> Result<Option<std::collections::HashSet<String>>> {
    match std::env::var_os("BENCH_SELECTION") {
        Some(file) => Ok(Some(serde_json::from_slice(&std::fs::read(file)?)?)),
        None => Ok(None),
    }
}
