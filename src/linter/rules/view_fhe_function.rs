// FHEVM-003: Function marked view/pure but calls a TFHE operation.
// FHE ops write to internal state registers — they cannot be view functions.

use regex::Regex;
use std::{path::Path, sync::OnceLock};
use super::{LintError, LintRule, Severity, make_error};

static FN_DECL_RE: OnceLock<Regex> = OnceLock::new();
static TFHE_OP_RE: OnceLock<Regex> = OnceLock::new();

pub struct ViewFheFunction;

impl LintRule for ViewFheFunction {
    fn id(&self) -> &'static str { "FHEVM-003" }

    fn check(&self, file_path: &Path, source: &str) -> Vec<LintError> {
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "sol" { return vec![]; }

        let fn_decl_re = FN_DECL_RE.get_or_init(|| {
            Regex::new(r"function\s+(\w+)\s*\([^)]*\)[^{]*\b(view|pure)\b[^{]*\{").unwrap()
        });
        let tfhe_op_re = TFHE_OP_RE.get_or_init(|| {
            Regex::new(r"TFHE\.(add|sub|mul|div|lt|le|gt|ge|eq|select|and|or|not|asEuint)").unwrap()
        });

        let lines: Vec<&str> = source.lines().collect();
        let mut errors = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if let Some(cap) = fn_decl_re.captures(line) {
                let fn_name = &cap[1];

                let body_end = (i + 50).min(lines.len());
                let body = lines[i..body_end].join("\n");

                if tfhe_op_re.is_match(&body) {
                    errors.push(make_error(
                        "FHEVM-003",
                        Severity::Error,
                        file_path,
                        i + 1,
                        &format!(
                            "Function '{}' is marked 'view' or 'pure' but calls a TFHE operation. \
                             FHE operations modify internal state and cannot be view functions. \
                             Remove the 'view' or 'pure' modifier.",
                            fn_name
                        ),
                        Some(line),
                    ));
                }
            }
        }

        errors
    }
}
