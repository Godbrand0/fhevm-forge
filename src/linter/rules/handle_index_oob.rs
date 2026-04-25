// FHEVM-008: handles[N] accessed where N >= known getter return count.
//
// getPositionHandles() returns exactly 2 values (collateral, debt).
// Accessing handles[2] or higher is always undefined — the health check
// handle is a separate getter (getPendingHealthHandle).

use regex::Regex;
use std::{path::Path, sync::OnceLock};
use super::{LintError, LintRule, Severity, make_error};

static HANDLES_INDEX_RE: OnceLock<Regex> = OnceLock::new();

pub struct HandleIndexOob;

impl LintRule for HandleIndexOob {
    fn id(&self) -> &'static str { "FHEVM-008" }

    fn check(&self, file_path: &Path, source: &str) -> Vec<LintError> {
        // Apply to both TypeScript and Solidity
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "ts" && ext != "tsx" && ext != "sol" { return vec![]; }

        // Matches: handles[2], handles[3], ... (index >= 2 is suspicious for 2-tuple getters)
        let re = HANDLES_INDEX_RE.get_or_init(|| {
            Regex::new(r"\bhandles\[([2-9]|\d{2,})\]").unwrap()
        });

        source
            .lines()
            .enumerate()
            .filter(|(_, line)| re.is_match(line))
            .map(|(i, line)| {
                let cap = re.captures(line).unwrap();
                let idx: usize = cap[1].parse().unwrap_or(0);
                make_error(
                    "FHEVM-008",
                    Severity::Error,
                    file_path,
                    i + 1,
                    &format!(
                        "handles[{}] accessed — getPositionHandles() returns only 2 values (collateral, debt). \
                         Use named destructuring instead of index access. The health check handle \
                         is a separate getter: getPendingHealthHandle(borrower).",
                        idx
                    ),
                    Some(line),
                )
            })
            .collect()
    }
}
