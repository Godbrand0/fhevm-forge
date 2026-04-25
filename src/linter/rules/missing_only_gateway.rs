// FHEVM-006: Gateway callback function missing onlyGateway modifier.
// Any address could call an unprotected callback and inject fake decryption results.

use regex::Regex;
use std::{path::Path, sync::OnceLock};
use super::{LintError, LintRule, Severity, make_error};

static CALLBACK_RE:     OnceLock<Regex> = OnceLock::new();
static ONLY_GATEWAY_RE: OnceLock<Regex> = OnceLock::new();

pub struct MissingOnlyGateway;

impl LintRule for MissingOnlyGateway {
    fn id(&self) -> &'static str { "FHEVM-006" }

    fn check(&self, file_path: &Path, source: &str) -> Vec<LintError> {
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "sol" { return vec![]; }

        let callback_re = CALLBACK_RE.get_or_init(|| {
            Regex::new(
                r"function\s+(_\w+Decrypted|_on\w+|callbackDecrypt\w*)\s*\([^)]*uint256[^)]*\)"
            ).unwrap()
        });
        let only_gateway_re = ONLY_GATEWAY_RE.get_or_init(|| {
            Regex::new(r"\bonlyGateway\b").unwrap()
        });

        let lines: Vec<&str> = source.lines().collect();
        let mut errors = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if callback_re.is_match(line) && !only_gateway_re.is_match(line) {
                errors.push(make_error(
                    "FHEVM-006",
                    Severity::Error,
                    file_path,
                    i + 1,
                    "Gateway callback function is missing the 'onlyGateway' modifier. \
                     Without this, any address can call the callback and inject arbitrary \
                     decryption results. Add 'onlyGateway' or inherit GatewayCallbackReceiver.",
                    Some(line),
                ));
            }
        }

        errors
    }
}
