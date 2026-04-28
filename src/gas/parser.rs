// Parses forge test --gas-report output and counts FHE op occurrences in source files.

use anyhow::Result;
use regex::Regex;
use std::{collections::HashMap, fs};
use walkdir::WalkDir;

/// Count TFHE operation calls in all .sol files under `path`.
pub fn count_fhe_ops(path: &str) -> Result<HashMap<String, u64>> {
    let mut counts: HashMap<String, u64> = HashMap::new();

    if !std::path::Path::new(path).exists() {
        return Ok(counts);
    }

    let op_re = Regex::new(
        r"TFHE\.(add|sub|mul|div|lt|le|gt|ge|eq|select|and|or|not|asEuint64|asEuint128|allowThis|allow)|Gateway\.requestDecryption"
    )?;

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "sol").unwrap_or(false))
    {
        let source = fs::read_to_string(entry.path())?;
        for cap in op_re.find_iter(&source) {
            *counts.entry(cap.as_str().to_string()).or_insert(0) += 1;
        }
    }

    Ok(counts)
}

/// Extract the gas report table section from raw forge output.
pub fn extract_gas_table(forge_output: &str) -> Option<&str> {
    let start = forge_output.find("╭")?;
    let end   = forge_output.rfind("╯").map(|i| i + 1)?;
    Some(&forge_output[start..end])
}
