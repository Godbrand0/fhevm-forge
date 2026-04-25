// FHEVM-007: .getRelayer() called — this method does not exist on FhevmInstance.
// Developers assume fhe.getRelayer().publicDecrypt() is the API.
// The correct API is fhe.publicDecrypt() directly.
// This rule scans TypeScript files (*.ts, *.tsx) not Solidity.

use regex::Regex;
use std::{path::Path, sync::OnceLock};
use super::{LintError, LintRule, Severity, make_error};

static GET_RELAYER_RE: OnceLock<Regex> = OnceLock::new();

pub struct GetRelayerCall;

impl LintRule for GetRelayerCall {
    fn id(&self) -> &'static str { "FHEVM-007" }

    fn check(&self, file_path: &Path, source: &str) -> Vec<LintError> {
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "ts" && ext != "tsx" { return vec![]; }

        let re = GET_RELAYER_RE.get_or_init(|| {
            Regex::new(r"\.getRelayer\(\)").unwrap()
        });

        source
            .lines()
            .enumerate()
            .filter(|(_, line)| re.is_match(line))
            .map(|(i, line)| {
                make_error(
                    "FHEVM-007",
                    Severity::Error,
                    file_path,
                    i + 1,
                    "Called .getRelayer() on FhevmInstance — this method does not exist. \
                     Use fhe.publicDecrypt() directly instead of fhe.getRelayer().publicDecrypt().",
                    Some(line),
                )
            })
            .collect()
    }
}
