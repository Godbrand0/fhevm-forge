// FHEVM-001: euint* assigned but TFHE.allowThis() not called in same block.
//
// Pattern: Find lines where a euint variable is assigned (either from
// TFHE.asEuint* or TFHE.add/sub/mul/etc.) but TFHE.allowThis() is not
// called within the next 5 lines.
//
// This is intentionally conservative — prefer false negatives over false
// positives. Only flag cases where assignment is obvious.

use regex::Regex;
use std::{path::Path, sync::OnceLock};
use super::{LintError, LintRule, Severity, make_error};

static ASSIGNMENT_RE: OnceLock<Regex> = OnceLock::new();
static ALLOW_THIS_RE: OnceLock<Regex> = OnceLock::new();

pub struct MissingAllowThis;

impl LintRule for MissingAllowThis {
    fn id(&self) -> &'static str { "FHEVM-001" }

    fn check(&self, file_path: &Path, source: &str) -> Vec<LintError> {
        // Only check Solidity files
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "sol" { return vec![]; }

        let assignment_re = ASSIGNMENT_RE.get_or_init(|| {
            Regex::new(r"euint\d+\s+(\w+)\s*=\s*TFHE\.").unwrap()
        });
        let allow_this_re = ALLOW_THIS_RE.get_or_init(|| {
            Regex::new(r"TFHE\.allowThis\(").unwrap()
        });

        let lines: Vec<&str> = source.lines().collect();
        let mut errors = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if let Some(cap) = assignment_re.captures(line) {
                let var_name = &cap[1];

                let window_end = (i + 6).min(lines.len());
                let window = &lines[i..window_end];
                let has_allow_this = window.iter().any(|l| allow_this_re.is_match(l));

                if !has_allow_this {
                    errors.push(make_error(
                        "FHEVM-001",
                        Severity::Error,
                        file_path,
                        i + 1,
                        &format!(
                            "euint variable '{}' assigned but TFHE.allowThis() not called in same scope. \
                             The contract cannot use this handle in future transactions.",
                            var_name
                        ),
                        Some(line),
                    ));
                }
            }
        }

        errors
    }
}
