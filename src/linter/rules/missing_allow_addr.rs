// FHEVM-002: euint* handle passed to external contract without TFHE.allow().
//
// Pattern: Detect when an external contract call is made with a euint argument
// but TFHE.allow(handle, address) is not called in the preceding lines.

use regex::Regex;
use std::{path::Path, sync::OnceLock};
use super::{LintError, LintRule, Severity, make_error};

static EXTERNAL_CALL_RE: OnceLock<Regex> = OnceLock::new();
static ALLOW_RE:         OnceLock<Regex> = OnceLock::new();

pub struct MissingAllowAddr;

impl LintRule for MissingAllowAddr {
    fn id(&self) -> &'static str { "FHEVM-002" }

    fn check(&self, file_path: &Path, source: &str) -> Vec<LintError> {
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "sol" { return vec![]; }

        // Matches patterns like: someContract.someMethod(euintVar, ...)
        // where the argument is a euint variable being passed externally
        let external_re = EXTERNAL_CALL_RE.get_or_init(|| {
            Regex::new(r"\b(\w+)\.(mint|transfer|deposit|withdraw|set|update|add|burn)\s*\([^)]*\b(euint|ebool)\b").unwrap()
        });
        let allow_re = ALLOW_RE.get_or_init(|| {
            Regex::new(r"TFHE\.allow\(").unwrap()
        });

        let lines: Vec<&str> = source.lines().collect();
        let mut errors = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if external_re.is_match(line) {
                // Look back 5 lines for TFHE.allow()
                let window_start = i.saturating_sub(5);
                let window = &lines[window_start..=i];
                let has_allow = window.iter().any(|l| allow_re.is_match(l));

                if !has_allow {
                    errors.push(make_error(
                        "FHEVM-002",
                        Severity::Error,
                        file_path,
                        i + 1,
                        "euint handle passed to external contract without TFHE.allow(). \
                         Call TFHE.allow(handle, address(externalContract)) before the external call.",
                        Some(line),
                    ));
                }
            }
        }

        errors
    }
}
