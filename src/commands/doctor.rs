use anyhow::Result;
use colored::Colorize;
use std::env;

pub async fn run() -> Result<()> {
    println!("\n{}\n", "Checking FHEVM development environment...".cyan().bold());

    let mut all_ok = true;

    let forge_ok = check_tool("forge", &["--version"]).await;
    print_check("forge", forge_ok, "Foundry not installed. Run: curl -L https://foundry.paradigm.xyz | bash");
    all_ok &= forge_ok;

    let node_ok = check_tool("node", &["--version"]).await;
    print_check("node >= 20", node_ok, "Node.js not installed. Required for TypeScript SDK. Install from https://nodejs.org");
    all_ok &= node_ok;

    let fhevm_installed = std::path::Path::new("lib/forge-fhevm").exists();
    print_check(
        "forge-fhevm",
        fhevm_installed,
        "forge-fhevm not installed. Run: forge install zama-ai/forge-fhevm --no-commit"
    );

    let foundry_ok = check_foundry_toml();
    print_check(
        "foundry.toml evm_version=cancun",
        foundry_ok,
        "foundry.toml missing or evm_version != cancun. forge-fhevm requires cancun EVM."
    );

    println!();

    let env_vars = [
        ("SEPOLIA_RPC_URL",      true,  "Required for Sepolia deployment"),
        ("DEPLOYER_PRIVATE_KEY", true,  "Required for all deployments"),
        ("MAINNET_RPC_URL",      false, "Required for mainnet deployment"),
        ("ETHERSCAN_API_KEY",    false, "Required for contract verification"),
        ("BASESCAN_API_KEY",     false, "Required for Base verification"),
    ];

    for (var, required, hint) in &env_vars {
        let is_set = env::var(var).map(|v| !v.is_empty()).unwrap_or(false);
        let label = format!("{:<30}", var);

        if is_set {
            println!("  {} {}", "✅".green(), label.green());
        } else if *required {
            println!("  {} {} — {}", "❌".red(), label.red(), hint.dimmed());
            all_ok = false;
        } else {
            println!(
                "  {} {} — {} {}",
                "⚠️ ".yellow(),
                label.yellow(),
                "not set".dimmed(),
                format!("({})", hint).dimmed()
            );
        }
    }

    println!();
    if all_ok {
        println!("{}", "✅ Environment is ready for FHEVM development.".green().bold());
    } else {
        println!("{}", "❌ Fix the issues above before deploying.".red().bold());
    }

    Ok(())
}

async fn check_tool(tool: &str, args: &[&str]) -> bool {
    tokio::process::Command::new(tool)
        .args(args)
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn print_check(label: &str, ok: bool, hint: &str) {
    let label = format!("{:<40}", label);
    if ok {
        println!("  {} {}", "✅".green(), label.green());
    } else {
        println!("  {} {} — {}", "❌".red(), label.red(), hint.dimmed());
    }
}

fn check_foundry_toml() -> bool {
    std::fs::read_to_string("foundry.toml")
        .map(|s| s.contains("cancun"))
        .unwrap_or(false)
}
