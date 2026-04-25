// FHEVM-005: Same inputProof variable passed to multiple TFHE.asEuint*() calls.
// inputProof is single-use and bound to one contract + user address pair.
// Reusing it across multiple asEuint calls will cause the second call to revert.

use regex::Regex;
use std::{collections::HashMap, path::Path, sync::OnceLock};
use super::{LintError, LintRule, Severity, make_error};

static AS_EUINT_RE: OnceLock<Regex> = OnceLock::new();

pub struct InputProofReuse;

impl LintRule for InputProofReuse {
    fn id(&self) -> &'static str { "FHEVM-005" }

    fn check(&self, file_path: &Path, source: &str) -> Vec<LintError> {
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "sol" { return vec![]; }

        // Matches TFHE.asEuint*(someVar, proofVar) capturing the proof argument
        let re = AS_EUINT_RE.get_or_init(|| {
            Regex::new(r"TFHE\.asEuint\w+\s*\(\s*\w+\s*,\s*(\w+)\s*\)").unwrap()
        });

        let lines: Vec<&str> = source.lines().collect();
        let mut errors = Vec::new();

        // Per-function scan: reset proof usage counter at each function boundary
        let mut proof_usage: HashMap<String, Vec<usize>> = HashMap::new();
        let mut in_function = false;

        for (i, line) in lines.iter().enumerate() {
            if line.contains("function ") && line.contains('{') {
                in_function = true;
                proof_usage.clear();
            }
            if in_function {
                if let Some(cap) = re.captures(line) {
                    let proof_var = cap[1].to_string();
                    let uses = proof_usage.entry(proof_var.clone()).or_default();
                    uses.push(i + 1);

                    if uses.len() == 2 {
                        errors.push(make_error(
                            "FHEVM-005",
                            Severity::Error,
                            file_path,
                            i + 1,
                            &format!(
                                "inputProof variable '{}' passed to multiple TFHE.asEuint*() calls. \
                                 inputProof is single-use — create separate encrypted inputs for each value.",
                                proof_var
                            ),
                            Some(line),
                        ));
                    }
                }
            }
        }

        errors
    }
}
