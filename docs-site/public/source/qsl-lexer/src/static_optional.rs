//! Static optional-layout graph. No per-message offset catalogue is built.
use crate::{static_index::PredictPlan, QslError, QslResult, QslSchema, QslType};
#[derive(Debug, Clone)]
pub enum StaticPresence {
    Eof,
    Length(PredictPlan),
    Mask { bytes: usize },
    Graph(crate::length_graph::LengthGraph),
}
#[derive(Debug, Clone)]
pub struct StaticOptionalPlan {
    pub presence: StaticPresence,
    pub fields: usize,
}
impl StaticOptionalPlan {
    pub fn analyze(schema: &QslSchema) -> QslResult<Self> {
        let fields = schema.optional_count();
        if fields > 64 || schema.compact() || schema.bitmap() {
            return Err(QslError::InvalidSchema);
        }
        if fields == 0 {
            return Ok(Self {
                presence: StaticPresence::Mask { bytes: 0 },
                fields,
            });
        }
        // Length intervals also resolve parts of a colliding layout. Native
        // string-length metadata is outside the payload index I.
        if let Some(graph) = crate::length_graph::LengthGraph::analyze(schema)? {
            return Ok(Self {
                presence: StaticPresence::Graph(graph),
                fields,
            });
        }
        if fields == 1 {
            let f = schema
                .fields
                .iter()
                .find(|f| f.optional)
                .ok_or(QslError::InvalidSchema)?;
            let positive = match &f.data {
                QslType::String(_)
                | QslType::RangedString { .. }
                | QslType::BorrowedString { .. }
                | QslType::InlineRawString { .. }
                | QslType::CompressedString { .. }
                | QslType::Array(_, _)
                | QslType::RangedArray { .. } => true,
                other => other.fixed_wire_size()?.is_some_and(|n| n > 0),
            };
            if positive {
                return Ok(Self {
                    presence: StaticPresence::Eof,
                    fields,
                });
            }
        }
        if let Ok(plan) = PredictPlan::analyze(schema) {
            if !plan.rows.is_empty() {
                return Ok(Self {
                    presence: StaticPresence::Length(plan),
                    fields,
                });
            }
        }
        Ok(Self {
            presence: StaticPresence::Mask {
                bytes: (fields + 7) >> 3,
            },
            fields,
        })
    }
    pub fn header_bytes(&self) -> usize {
        match self.presence {
            StaticPresence::Mask { bytes } => bytes,
            StaticPresence::Graph(ref p) => p.metadata_bytes,
            _ => 0,
        }
    }
}
