// Terminal, JSON and Markdown output for the FHE gas report.

use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;

use crate::gas::costs::{FheType, fhe_cost};

pub fn terminal(counts: &HashMap<(String, FheType), u64>) {
    println!("\n{}", "⛽ FHE Gas Report".cyan().bold());
    println!("{}", "─".repeat(91));
    println!(
        "{:<36} {:<10} {:<18} {:<18} {}",
        "Operation".bold(), "Count".bold(),
        "On-Chain Gas".bold(), "Coprocessor Gas".bold(), "% of FHE Total".bold()
    );
    println!("{}", "─".repeat(91));

    let mut rows: Vec<(String, u64, u64, u64)> = counts
        .iter()
        .map(|((op, ty), &count)| {
            let label = format!("{} ({})", op, ty.label());
            let (on_chain, cop) = fhe_cost(op, *ty);
            (label, count, on_chain * count, cop * count)
        })
        .collect();

    let total: u64 = rows.iter().map(|(_, _, on, cop)| on + cop).sum();
    rows.sort_by(|a, b| (b.2 + b.3).cmp(&(a.2 + a.3)));

    for (label, count, on_chain, cop) in &rows {
        let pct = if total > 0 {
            ((on_chain + cop) as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        println!("{:<36} {:<10} {:<18} {:<18} {:.1}%", label, count, fmt(on_chain), fmt(cop), pct);
    }

    println!("{}", "─".repeat(91));
    let total_on: u64  = rows.iter().map(|(_, _, on, _)| on).sum();
    let total_cop: u64 = rows.iter().map(|(_, _, _, c)| c).sum();
    println!(
        "{:<36} {:<10} {:<18} {:<18}",
        "TOTAL".bold(), "",
        fmt(&total_on).bold().to_string(),
        fmt(&total_cop).bold().to_string()
    );

    let baseline: u64 = rows.iter().map(|(_, count, _, _)| count * 100).sum();
    if baseline > 0 {
        let mult = (total_on + total_cop) as f64 / baseline as f64;
        println!("\n{} FHE total is ~{:.1}x the cost of equivalent plaintext EVM logic", "⚠️ ".yellow(), mult);
    }
    if let Some((label, _, _, _)) = rows.first() {
        println!("{} Most expensive: {} — consider batching or reducing call count", "💡".cyan(), label.yellow());
    }
}

pub fn json(counts: &HashMap<(String, FheType), u64>) -> Result<()> {
    let report: Vec<serde_json::Value> = counts
        .iter()
        .map(|((op, ty), &count)| {
            let (on_chain, cop) = fhe_cost(op, *ty);
            serde_json::json!({
                "operation": op,
                "type": ty.label(),
                "count": count,
                "on_chain_gas": on_chain * count,
                "coprocessor_gas": cop * count,
                "total_gas": (on_chain + cop) * count,
            })
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

pub fn markdown(counts: &HashMap<(String, FheType), u64>) {
    println!("## FHE Gas Report\n");
    println!("| Operation | Type | Count | On-Chain Gas | Coprocessor Gas |");
    println!("|-----------|------|-------|-------------|-----------------|");
    let mut rows: Vec<_> = counts.iter().collect();
    rows.sort_by_key(|((op, ty), _)| (op.clone(), *ty));
    for ((op, ty), &count) in rows {
        let (on_chain, cop) = fhe_cost(op, *ty);
        println!("| {} | {} | {} | {} | {} |", op, ty.label(), count, on_chain * count, cop * count);
    }
}

fn fmt(gas: &u64) -> String {
    if *gas == 0 { return "0".to_string(); }
    let s = gas.to_string();
    let mut r = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { r.push(','); }
        r.push(c);
    }
    r.chars().rev().collect()
}
