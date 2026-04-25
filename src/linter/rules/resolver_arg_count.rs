// FHEVM-009: Resolver function called with fewer than 3 required arguments.
//
// Contract resolver functions (resolveHealthCheck, resolveBid, etc.) require 3 args:
//   1. The identifying key (borrower address, auctionId, etc.)
//   2. abiEncodedClearValues (from publicDecrypt)
//   3. decryptionProof       (from publicDecrypt)
//
// Calling with only 1 arg will revert silently.

use regex::Regex;
use std::{path::Path, sync::OnceLock};
use super::{LintError, LintRule, Severity, make_error};

static RESOLVE_ONE_ARG_RE: OnceLock<Regex> = OnceLock::new();

pub struct ResolverArgCount;

impl LintRule for ResolverArgCount {
    fn id(&self) -> &'static str { "FHEVM-009" }

    fn check(&self, file_path: &Path, source: &str) -> Vec<LintError> {
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "ts" && ext != "tsx" && ext != "sol" { return vec![]; }

        // Matches resolve* functions called with only one argument (no comma = 1 arg)
        // e.g.: resolveHealthCheck(borrower) — missing abiEncoded + proof
        let re = RESOLVE_ONE_ARG_RE.get_or_init(|| {
            Regex::new(r"\bresolve\w*\s*\(\s*[^,)]+\s*\)").unwrap()
        });

        source
            .lines()
            .enumerate()
            .filter(|(_, line)| re.is_match(line))
            .map(|(i, line)| {
                make_error(
                    "FHEVM-009",
                    Severity::Error,
                    file_path,
                    i + 1,
                    "Resolver function called with only 1 argument. \
                     Contract resolvers require 3 args: (key, abiEncodedClearValues, decryptionProof). \
                     Pass both return values from publicDecrypt() to the resolver.",
                    Some(line),
                )
            })
            .collect()
    }
}
