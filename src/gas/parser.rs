// Parses forge test --gas-report output and counts FHE op occurrences in source files.

use anyhow::Result;
use regex::Regex;
use std::{collections::HashMap, fs};
use walkdir::WalkDir;

use crate::gas::costs::FheType;

/// Count TFHE operation calls in all .sol files under `path`, keyed by (op, ciphertext type).
pub fn count_fhe_ops(path: &str) -> Result<HashMap<(String, FheType), u64>> {
    let mut counts: HashMap<(String, FheType), u64> = HashMap::new();

    if !std::path::Path::new(path).exists() {
        return Ok(counts);
    }

    let op_re = Regex::new(
        r"(?:TFHE|FHE)\.(add|sub|mul|div|rem|neg|lt|le|gt|ge|eq|ne|select|and|or|xor|not|shl|shr|min|max|asEbool|asEuint8|asEuint16|asEuint32|asEuint64|asEuint128|asEuint256|fromExternal|allowThis|allow|makePubliclyDecryptable|isInitialized)|Gateway\.requestDecryption"
    )?;

    // Type declaration regex for variable names: captures (sol_type, var_name)
    let decl_re = Regex::new(
        r"\b(ebool|euint256|euint128|euint64|euint32|euint16|euint8)\s+(?:(?:public|private|internal|external|immutable|constant)\s+)*([a-zA-Z_]\w*)\b"
    )?;

    // Type keyword regex for same-line LHS detection (longest first to avoid prefix matches)
    let ty_kw_re = Regex::new(
        r"\b(euint256|euint128|euint64|euint32|euint16|euint8|ebool)\b"
    )?;

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "sol").unwrap_or(false))
    {
        let source = fs::read_to_string(entry.path())?;
        let var_types = extract_var_types(&source, &decl_re);

        for line in source.lines() {
            for cap in op_re.find_iter(line) {
                let op = cap.as_str().to_string();
                let bare = op.strip_prefix("TFHE.")
                    .or_else(|| op.strip_prefix("Gateway."))
                    .unwrap_or(&op);
                let ty = infer_op_type(bare, line, &var_types, &ty_kw_re);
                *counts.entry((op, ty)).or_insert(0) += 1;
            }
        }
    }

    Ok(counts)
}

/// Build a map of variable name → FheType from all declarations in a source file.
fn extract_var_types(source: &str, decl_re: &Regex) -> HashMap<String, FheType> {
    let mut map = HashMap::new();
    for cap in decl_re.captures_iter(source) {
        if let Some(ty) = FheType::from_sol(&cap[1]) {
            map.entry(cap[2].to_string()).or_insert(ty);
        }
    }
    map
}

/// Infer the ciphertext type for an operation using line context and variable declarations.
fn infer_op_type(
    bare_op: &str,
    line: &str,
    var_types: &HashMap<String, FheType>,
    ty_kw_re: &Regex,
) -> FheType {
    // Cast ops encode their destination type in the name.
    if bare_op.starts_with("as") {
        return match bare_op {
            "asEbool"    => FheType::Bool,
            "asEuint8"   => FheType::Uint8,
            "asEuint16"  => FheType::Uint16,
            "asEuint32"  => FheType::Uint32,
            "asEuint64"  => FheType::Uint64,
            "asEuint128" => FheType::Uint128,
            "asEuint256" => FheType::Uint256,
            _            => FheType::Uint64,
        };
    }

    let is_comparison = matches!(bare_op, "lt"|"le"|"gt"|"ge"|"eq"|"ne");

    // For arithmetic/bitwise/select ops the result type matches the operand type.
    // Check for an explicit type keyword on the same line (LHS declaration).
    if !is_comparison {
        if let Some(cap) = ty_kw_re.find(line) {
            if let Some(ty) = FheType::from_sol(cap.as_str()) {
                return ty;
            }
        }
    }

    // Try to resolve the first argument name against the known variable types.
    let search = format!(".{}(", bare_op);
    if let Some(pos) = line.find(&search) {
        let after = line[pos + search.len()..].trim_start();
        let arg: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if let Some(&ty) = var_types.get(&arg) {
            return ty;
        }
    }

    FheType::Uint64
}

/// Extract the gas report table section from raw forge output.
pub fn extract_gas_table(forge_output: &str) -> Option<&str> {
    let start = forge_output.find("╭")?;
    let end   = forge_output.rfind("╯").map(|i| i + '╯'.len_utf8())?;
    Some(&forge_output[start..end])
}
