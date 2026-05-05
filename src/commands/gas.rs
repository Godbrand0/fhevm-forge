use anyhow::Result;
use colored::Colorize;
use crate::gas::{costs::fhe_cost, parser::{count_fhe_ops, extract_gas_table}, reporter};
use crate::config::FhevmForgeConfig;

pub async fn run(contract: Option<&str>, output_format: &str) -> Result<()> {
    let cfg = FhevmForgeConfig::load()?;

    println!("{} forge test --gas-report...", "Running".cyan().bold());

    let mut cmd = tokio::process::Command::new("forge");
    cmd.args(["test", "--gas-report"]);
    if let Some(c) = contract {
        cmd.args(["--match-contract", c]);
    }

    let output = cmd.output().await;
    let forge_stdout = match output {
        Ok(ref o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => {
            eprintln!("{} forge not found — skipping forge gas report", "⚠️ ".yellow());
            String::new()
        }
    };

    let op_counts = count_fhe_ops("./src")?;

    match output_format {
        "json"     => reporter::json(&op_counts)?,
        "markdown" => reporter::markdown(&op_counts),
        _          => reporter::terminal(&op_counts),
    }

    if let Some(threshold) = cfg.gas.warn_if_coprocessor_gas_exceeds {
        let total_cop: u64 = op_counts
            .iter()
            .map(|((op, ty), &count)| fhe_cost(op, *ty).1 * count)
            .sum();
        if total_cop > threshold {
            println!(
                "\n{} Total coprocessor gas ({}) exceeds configured threshold ({})",
                "⚠️ ".yellow(),
                total_cop,
                threshold
            );
        }
    }

    if let Some(table) = extract_gas_table(&forge_stdout) {
        println!("\n{}", "Standard Forge Gas Report:".dimmed());
        println!("{}", table.dimmed());
    }

    Ok(())
}
