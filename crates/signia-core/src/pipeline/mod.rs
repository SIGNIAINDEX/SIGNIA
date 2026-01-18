//! Compiler pipeline primitives for SIGNIA.
//!
//! The SIGNIA ecosystem typically has multiple "producers":
//! - CLI (`signia compile ...`)
//! - API service (`POST /v1/compile`)
//! - CI workflows
//! - plugins (internal stages)
//!
//! To avoid duplicated logic, SIGNIA uses a pipeline abstraction:
//! - inputs are gathered and normalized
//! - plugins generate an IR graph
//! - the IR is normalized and validated
//! - the IR is emitted to SchemaV1/ManifestV1/ProofV1
//! - optional verification replays the deterministic computations
//!
//! This module defines:
//! - `Pipeline` and `Stage` traits
//! - `PipelineContext` (deterministic config, tracing, diagnostics)
//! - `PipelineReport` (structured outputs + warnings)
//!
//! The core crate does not do network or filesystem I/O. Higher-level crates
//! perform I/O and pass bytes/structures into the pipeline.

use std::collections::BTreeMap;

use crate::errors::{SigniaError, SigniaResult};

#[cfg(feature = "canonical-json")]
use serde_json::Value;

pub mod stages;

/// A stable identifier for a pipeline stage.
///
/// Use dot-delimited namespaces:
/// - `input.normalize`
/// - `plugin.repo`
/// - `ir.validate`
/// - `emit.schema_v1`
/// - `proof.merkle`
pub type StageId = String;

/// A structured diagnostic emitted by pipeline stages.
///
/// Diagnostics are intended for:
/// - CLI printing
/// - API response payloads
/// - Console display
#[derive(Debug, Clone)]
pub struct PipelineDiagnostic {
    pub level: DiagnosticLevel,
    pub code: String,
    pub message: String,
    pub data: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy)]
pub enum DiagnosticLevel {
    Info,
    Warning,
    Error,
}

/// A deterministic clock abstraction.
///
/// Core does not read system time. If a stage needs a timestamp,
/// the higher-level caller must inject it via the context.
#[derive(Debug, Clone)]
pub struct DeterministicClock {
    /// An ISO-8601 timestamp chosen by the caller.
    pub now_iso8601: String,
}

impl Default for DeterministicClock {
    fn default() -> Self {
        Self {
            now_iso8601: "1970-01-01T00:00:00Z".to_string(),
        }
    }
}

/// Pipeline context shared by all stages.
///
/// This is the main mechanism for passing:
/// - deterministic configuration
/// - stage parameters
/// - compiler hints
/// - diagnostics collection
#[derive(Debug, Clone)]
pub struct PipelineContext {
    /// Deterministic clock values (no system reads).
    pub clock: DeterministicClock,

    /// Caller-defined parameters. Keys should be stable and documented.
    pub params: BTreeMap<String, String>,

    /// Optional JSON params for more complex configs (plugin configs).
    #[cfg(feature = "canonical-json")]
    pub json_params: BTreeMap<String, Value>,

    /// Collected diagnostics.
    pub diagnostics: Vec<PipelineDiagnostic>,
}

impl Default for PipelineContext {
    fn default() -> Self {
        Self {
            clock: DeterministicClock::default(),
            params: BTreeMap::new(),
            #[cfg(feature = "canonical-json")]
            json_params: BTreeMap::new(),
            diagnostics: Vec::new(),
        }
    }
}

impl PipelineContext {
    pub fn push_info(&mut self, code: impl Into<String>, message: impl Into<String>) {
        self.diagnostics.push(PipelineDiagnostic {
            level: DiagnosticLevel::Info,
            code: code.into(),
            message: message.into(),
            data: BTreeMap::new(),
        });
    }

    pub fn push_warning(&mut self, code: impl Into<String>, message: impl Into<String>) {
        self.diagnostics.push(PipelineDiagnostic {
            level: DiagnosticLevel::Warning,
            code: code.into(),
            message: message.into(),
            data: BTreeMap::new(),
        });
    }

    pub fn push_error(&mut self, code: impl Into<String>, message: impl Into<String>) {
        self.diagnostics.push(PipelineDiagnostic {
            level: DiagnosticLevel::Error,
            code: code.into(),
            message: message.into(),
            data: BTreeMap::new(),
        });
    }

    pub fn set_param(&mut self, k: impl Into<String>, v: impl Into<String>) {
        self.params.insert(k.into(), v.into());
    }

    pub fn get_param(&self, k: &str) -> Option<&str> {
        self.params.get(k).map(|s| s.as_str())
    }

    #[cfg(feature = "canonical-json")]
    pub fn set_json_param(&mut self, k: impl Into<String>, v: Value) {
        self.json_params.insert(k.into(), v);
    }

    #[cfg(feature = "canonical-json")]
    pub fn get_json_param(&self, k: &str) -> Option<&Value> {
        self.json_params.get(k)
    }
}

/// A stage input/output carrier.
///
/// Stages may operate on different data shapes. To keep the pipeline generic,
/// we use a small enum that can be extended.
#[derive(Debug, Clone)]
pub enum PipelineData {
    None,

    /// Canonical JSON values (for models, plugin config, etc).
    #[cfg(feature = "canonical-json")]
    Json(Value),

    /// Bytes (for canonical bundles or exported artifacts).
    Bytes(Vec<u8>),

    /// IR graph (internal compilation representation).
    #[cfg(feature = "canonical-json")]
    Ir(crate::model::ir::IrGraph),

    /// Schema v1.
    #[cfg(feature = "canonical-json")]
    SchemaV1(crate::model::v1::SchemaV1),

    /// Manifest v1.
    #[cfg(feature = "canonical-json")]
    ManifestV1(crate::model::v1::ManifestV1),

    /// Proof v1.
    #[cfg(feature = "canonical-json")]
    ProofV1(crate::model::v1::ProofV1),
}

/// A pipeline stage.
///
/// Stages should be deterministic: do not read system time, env, random, network.
/// Any such values must be injected via `PipelineContext`.
pub trait Stage {
    fn id(&self) -> &str;
    fn run(&self, ctx: &mut PipelineContext, input: PipelineData) -> SigniaResult<PipelineData>;
}

/// A pipeline is an ordered list of stages.
#[derive(Debug, Default)]
pub struct Pipeline {
    stages: Vec<Box<dyn Stage + Send + Sync>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn push_stage<S: Stage + Send + Sync + 'static>(&mut self, s: S) -> &mut Self {
        self.stages.push(Box::new(s));
        self
    }

    pub fn stages(&self) -> usize {
        self.stages.len()
    }

    /// Run the pipeline and return a structured report.
    pub fn run(&self, mut ctx: PipelineContext, input: PipelineData) -> SigniaResult<PipelineReport> {
        let mut data = input;

        for st in &self.stages {
            ctx.push_info(
                "pipeline.stage.start",
                format!("starting stage {}", st.id()),
            );

            data = st.run(&mut ctx, data)?;

            ctx.push_info(
                "pipeline.stage.end",
                format!("completed stage {}", st.id()),
            );
        }

        Ok(PipelineReport {
            output: data,
            diagnostics: ctx.diagnostics,
        })
    }
}

/// Pipeline run result.
#[derive(Debug)]
pub struct PipelineReport {
    pub output: PipelineData,
    pub diagnostics: Vec<PipelineDiagnostic>,
}

impl PipelineReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| matches!(d.level, DiagnosticLevel::Error))
    }

    pub fn warnings(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| matches!(d.level, DiagnosticLevel::Warning))
            .count()
    }
}

/// A convenience helper to require a specific output variant from a report.
impl PipelineReport {
    #[cfg(feature = "canonical-json")]
    pub fn require_schema_v1(self) -> SigniaResult<crate::model::v1::SchemaV1> {
        match self.output {
            PipelineData::SchemaV1(s) => Ok(s),
            other => Err(SigniaError::invalid_argument(format!(
                "expected SchemaV1 pipeline output, got {other:?}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PassThroughStage;
    impl Stage for PassThroughStage {
        fn id(&self) -> &str {
            "test.pass"
        }
        fn run(&self, _ctx: &mut PipelineContext, input: PipelineData) -> SigniaResult<PipelineData> {
            Ok(input)
        }
    }

    struct ErrorStage;
    impl Stage for ErrorStage {
        fn id(&self) -> &str {
            "test.error"
        }
        fn run(&self, ctx: &mut PipelineContext, _input: PipelineData) -> SigniaResult<PipelineData> {
            ctx.push_error("test.error", "forced error");
            Err(SigniaError::invariant("stage failed"))
        }
    }

    #[test]
    fn pipeline_runs_stages() {
        let mut p = Pipeline::new();
        p.push_stage(PassThroughStage);

        let report = p.run(PipelineContext::default(), PipelineData::Bytes(vec![1, 2, 3])).unwrap();
        match report.output {
            PipelineData::Bytes(b) => assert_eq!(b, vec![1, 2, 3]),
            _ => panic!("unexpected output"),
        }
        assert!(!report.has_errors());
    }

    #[test]
    fn pipeline_propagates_error() {
        let mut p = Pipeline::new();
        p.push_stage(ErrorStage);

        let r = p.run(PipelineContext::default(), PipelineData::None);
        assert!(r.is_err());
    }
}
