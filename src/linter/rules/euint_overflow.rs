// FHEVM-004: euint64 used in TFHE.mul() — possible overflow for large values.
// euint64 max is 2^64-1. Multiplying two large euint64 values will overflow silently.
// Consider using euint128 for operands in multiplication.

use regex::Regex;
use std::{path::Path, sync::OnceLock};
use super::{LintError, LintRule, Severity, make_error};

static MUL_EUINT64_RE: OnceLock<Regex> = OnceLock::new();

pub struct EuintOverflow;

impl LintRule for EuintOverflow {
    fn id(&self) -> &'static str { "FHEVM-004" }

    fn check(&self, file_path: &Path, source: &str) -> Vec<LintError> {
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "sol" { return vec![]; }

        // Detect TFHE.mul() where at least one argument looks like a euint64 variable
        // (heuristic: variable declared as euint64 earlier in the function)
        let re = MUL_EUINT64_RE.get_or_init(|| {
            Regex::new(r"TFHE\.mul\s*\(").unwrap()
        });

        let lines: Vec<&str> = source.lines().collect();
        let mut errors = Vec::new();

        // Track whether euint64 is declared in scope
        let mut has_euint64 = false;
        for (i, line) in lines.iter().enumerate() {
            if line.contains("euint64") {
                has_euint64 = true;
            }
            if has_euint64 && re.is_match(line) {
                errors.push(make_error(
                    "FHEVM-004",
                    Severity::Warning,
                    file_path,
                    i + 1,
                    "TFHE.mul() used with euint64 — possible overflow for values > 2^32. \
                     Consider using euint128 operands for multiplication to avoid silent overflow.",
                    Some(line),
                ));
            }
        }

        errors
    }
}
