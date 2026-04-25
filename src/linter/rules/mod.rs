use std::path::Path;

pub mod missing_allow_this;
pub mod missing_allow_addr;
pub mod view_fhe_function;
pub mod euint_overflow;
pub mod input_proof_reuse;
pub mod get_relayer_call;
pub mod missing_only_gateway;
pub mod handle_index_oob;
pub mod resolver_arg_count;

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct LintError {
    pub rule_id:  String,
    pub severity: Severity,
    pub file:     String,
    pub line:     usize,
    pub message:  String,
    pub snippet:  Option<String>,
}

pub trait LintRule: Send + Sync {
    fn id(&self) -> &'static str;
    fn check(&self, file_path: &Path, source: &str) -> Vec<LintError>;
}

pub fn make_error(
    rule_id:  &str,
    severity: Severity,
    file:     &Path,
    line:     usize,
    message:  &str,
    snippet:  Option<&str>,
) -> LintError {
    LintError {
        rule_id:  rule_id.to_string(),
        severity,
        file:     file.display().to_string(),
        line,
        message:  message.to_string(),
        snippet:  snippet.map(|s| s.trim().to_string()),
    }
}

// Re-export all rule types
pub use self::missing_allow_this::MissingAllowThis;
pub use self::missing_allow_addr::MissingAllowAddr;
pub use self::view_fhe_function::ViewFheFunction;
pub use self::euint_overflow::EuintOverflow;
pub use self::input_proof_reuse::InputProofReuse;
pub use self::get_relayer_call::GetRelayerCall;
pub use self::missing_only_gateway::MissingOnlyGateway;
pub use self::handle_index_oob::HandleIndexOob;
pub use self::resolver_arg_count::ResolverArgCount;
