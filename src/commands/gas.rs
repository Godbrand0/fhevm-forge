use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;

fn fhe_costs() -> HashMap<&'static str, (u64, u64)> {
    let mut m = HashMap::new();
    m.insert("TFHE.add",                    (8_000,  65_000));
    m.insert("TFHE.sub",                    (8_000,  65_000));
    m.insert("TFHE.mul",                    (15_000, 150_000));
    m.insert("TFHE.div",                    (30_000, 400_000));
    m.insert("TFHE.lt",                     (10_000, 70_000));
    m.insert("TFHE.le",                     (10_000, 70_000));
    m.insert("TFHE.gt",                     (10_000, 70_000));
    m.insert("TFHE.ge",                     (10_000, 70_000));
    m.insert("TFHE.eq",                     (10_000, 70_000));
    m.insert("TFHE.select",                 (12_000, 90_000));
    m.insert("TFHE.and",                    (5_000,  30_000));
    m.insert("TFHE.or",                     (5_000,  30_000));
    m.insert("TFHE.not",                    (5_000,  30_000));
    m.insert("TFHE.asEuint64",              (6_000,  50_000));
    m.insert("TFHE.asEuint128",             (6_500,  55_000));
    m.insert("TFHE.allow",                  (3_000,  0));
    m.insert("TFHE.allowThis",              (3_000,  0));
    m.insert("Gateway.requestDecryption",   (25_000, 200_000));
    m
}

pub async fn run(contract: Option<&str>, output_format: &str) -> Result<()> {
    println!("{} forge test --gas-report...", "Running".cyan().bold());

    let mut cmd = tokio::process::Command::new("forge");
    cmd.args(["test", "--gas-report"]);
    if let Some(c) = contract {
        cmd.args(["--match-contract", c]);
    }

    let output = cmd.output().await?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let op_counts = count_fhe_ops_in_source("./src")?;
    let costs = fhe_costs();

    match output_format {
        "json"     => print_json_report(&op_counts, &costs)?,
        "markdown" => print_markdown_report(&op_counts, &costs),
        _          => print_terminal_report(&op_counts, &costs),
    }

    if !stdout.is_empty() {
        println!("\n{}", "Standard Forge Gas Report:".dimmed());
        println!("{}", stdout.dimmed());
    }

    Ok(())
}

fn count_fhe_ops_in_source(path: &str) -> Result<HashMap<String, u64>> {
    use std::fs;
    use walkdir::WalkDir;
    use regex::Regex;

    let mut counts: HashMap<String, u64> = HashMap::new();
    let op_re = Regex::new(
        r"TFHE\.(add|sub|mul|div|lt|le|gt|ge|eq|select|and|or|not|asEuint64|asEuint128|allow|allowThis)|Gateway\.requestDecryption"
    )?;

    if !std::path::Path::new(path).exists() {
        return Ok(counts);
    }

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

fn print_terminal_report(counts: &HashMap<String, u64>, costs: &HashMap<&str, (u64, u64)>) {
    println!("\n{}", "⛽ FHE Gas Report".cyan().bold());
    println!("{}", "─".repeat(85));
    println!(
        "{:<30} {:<10} {:<18} {:<18} {}",
        "Operation".bold(),
        "Count".bold(),
        "On-Chain Gas".bold(),
        "Coprocessor Gas".bold(),
        "% of FHE Total".bold()
    );
    println!("{}", "─".repeat(85));

    let mut rows: Vec<(String, u64, u64, u64)> = counts
        .iter()
        .filter_map(|(op, &count)| {
            costs.get(op.as_str()).map(|&(on_chain, coprocessor)| {
                (op.clone(), count, on_chain * count, coprocessor * count)
            })
        })
        .collect();

    let total_fhe: u64 = rows.iter().map(|(_, _, on, cop)| on + cop).sum();

    rows.sort_by(|a, b| (b.2 + b.3).cmp(&(a.2 + a.3)));

    for (op, count, on_chain, coprocessor) in &rows {
        let pct = if total_fhe > 0 {
            ((on_chain + coprocessor) as f64 / total_fhe as f64) * 100.0
        } else { 0.0 };

        println!(
            "{:<30} {:<10} {:<18} {:<18} {:.1}%",
            op, count,
            format_gas(*on_chain),
            format_gas(*coprocessor),
            pct
        );
    }

    println!("{}", "─".repeat(85));

    let total_on_chain: u64    = rows.iter().map(|(_, _, on, _)| on).sum();
    let total_coprocessor: u64 = rows.iter().map(|(_, _, _, cop)| cop).sum();
    let evm_baseline: u64      = rows.iter().map(|(_, count, _, _)| count * 100).sum();

    println!(
        "{:<30} {:<10} {:<18} {:<18}",
        "TOTAL".bold(),
        "",
        format_gas(total_on_chain).bold().to_string(),
        format_gas(total_coprocessor).bold().to_string(),
    );

    if evm_baseline > 0 {
        let multiplier = (total_on_chain + total_coprocessor) as f64 / evm_baseline as f64;
        println!(
            "\n{} FHE total is ~{:.1}x the cost of equivalent plaintext EVM logic",
            "⚠️ ".yellow(),
            multiplier
        );
    }

    if let Some((most_expensive, _, _, _)) = rows.first() {
        println!(
            "{} Most expensive operation: {} — consider batching or reducing call count",
            "💡".cyan(),
            most_expensive.yellow()
        );
    }

    if rows.is_empty() {
        println!("\n{}", "No TFHE operations found in ./src. Run from your project root.".yellow());
    }
}

fn print_json_report(counts: &HashMap<String, u64>, costs: &HashMap<&str, (u64, u64)>) -> Result<()> {
    let report: Vec<serde_json::Value> = counts.iter()
        .filter_map(|(op, &count)| {
            costs.get(op.as_str()).map(|&(on_chain, coprocessor)| {
                serde_json::json!({
                    "operation": op,
                    "count": count,
                    "on_chain_gas": on_chain * count,
                    "coprocessor_gas": coprocessor * count,
                    "total_gas": (on_chain + coprocessor) * count,
                })
            })
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn print_markdown_report(counts: &HashMap<String, u64>, costs: &HashMap<&str, (u64, u64)>) {
    println!("## FHE Gas Report\n");
    println!("| Operation | Count | On-Chain Gas | Coprocessor Gas |");
    println!("|-----------|-------|-------------|-----------------|");
    for (op, &count) in counts {
        if let Some(&(on_chain, coprocessor)) = costs.get(op.as_str()) {
            println!("| {} | {} | {} | {} |", op, count, on_chain * count, coprocessor * count);
        }
    }
}

fn format_gas(gas: u64) -> String {
    if gas == 0 { return "0".to_string(); }
    let s = gas.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { result.push(','); }
        result.push(c);
    }
    result.chars().rev().collect()
}
