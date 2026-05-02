use anyhow::{Context, Result, bail};
use colored::Colorize;
use dialoguer::{Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use crate::scaffold::generator::Generator;
use crate::scaffold::templates::{TEMPLATES, is_valid_template};

const TEMPLATE_LABELS: &[(&str, &str)] = &[
    ("blank",   "Blank FHEVM Project (bare Foundry + forge-fhevm)"),
    ("erc7984", "Confidential ERC-7984 Token"),
    ("lending", "Confidential Lending Vault (Vault + cETH + cUSDC)"),
    ("auction", "Blind Dutch Auction"),
    ("voting",  "Confidential Voting System"),
];

pub async fn run(name: &str, template_flag: Option<&str>) -> Result<()> {
    if name.is_empty() {
        bail!("Project name cannot be empty");
    }

    let target = Path::new(name);
    if target.exists() {
        bail!("Directory '{}' already exists. Choose a different name.", name);
    }

    let template = match template_flag {
        Some(t) => {
            if !is_valid_template(t) {
                bail!("Unknown template '{}'. Valid options: {}", t, TEMPLATES.join(", "));
            }
            t.to_string()
        }
        None => {
            let labels: Vec<&str> = TEMPLATE_LABELS.iter().map(|(_, l)| *l).collect();
            let idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose a starting template")
                .items(&labels)
                .default(2)
                .interact()
                .context("Failed to show template selector")?;
            TEMPLATE_LABELS[idx].0.to_string()
        }
    };

    println!(
        "\n{} {} project in {}/\n",
        "Scaffolding".cyan().bold(),
        template.yellow(),
        name
    );

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    pb.set_message("Running forge init...");
    forge_init(name).await.context("forge init failed")?;

    pb.set_message("Installing zama-ai/forge-fhevm...");
    forge_install(name, "zama-ai/forge-fhevm").await
        .context("forge install zama-ai/forge-fhevm failed")?;

    pb.set_message("Installing forge-fhevm soldeer dependencies...");
    soldeer_install(name).await
        .context("forge soldeer install (inside lib/forge-fhevm) failed")?;

    pb.set_message("Generating contract and SDK files...");
    let generator = Generator::new(name, &template)?;
    generator.render_all().context("Template rendering failed")?;

    pb.set_message("Writing configuration files...");
    generator.write_config_files().context("Failed to write config files")?;

    pb.set_message("Installing npm dependencies...");
    npm_install(name).await.context("npm install failed")?;

    pb.finish_and_clear();

    println!("{}\n", "✅ Project scaffolded successfully!".green().bold());
    println!("  {} {}", "cd".dimmed(), name.cyan());
    println!("  {}  # Run FHE tests (uses forge-fhevm local mock)", "forge test".cyan());
    println!("  {}      # Estimate FHE gas costs", "fhevm-forge gas".cyan());
    println!("  {}     # Check for TFHE.allow() bugs", "fhevm-forge lint".cyan());
    println!("  {} --chains sepolia  # Deploy to testnet", "fhevm-forge deploy".cyan());
    println!("\n  Read {} for FHEVM development guidelines.", "AGENT.md".yellow());

    Ok(())
}

async fn forge_init(name: &str) -> Result<()> {
    let output = tokio::process::Command::new("forge")
        .args(["init", name, "--no-git"])
        .output()
        .await
        .context(
            "Could not find 'forge'. Install Foundry: https://getfoundry.sh\n\
             Run: curl -L https://foundry.paradigm.xyz | bash"
        )?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("forge init failed:\n{}", stderr);
    }
    Ok(())
}

async fn soldeer_install(project_dir: &str) -> Result<()> {
    let forge_fhevm_dir = Path::new(project_dir).join("lib/forge-fhevm");
    let output = tokio::process::Command::new("forge")
        .args(["soldeer", "install"])
        .current_dir(&forge_fhevm_dir)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("forge soldeer install failed:\n{}", stderr);
    }
    Ok(())
}

async fn forge_install(project_dir: &str, dep: &str) -> Result<()> {
    let output = tokio::process::Command::new("forge")
        .args(["install", dep, "--no-git"])
        .current_dir(project_dir)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("forge install {} failed:\n{}", dep, stderr);
    }
    Ok(())
}

async fn npm_install(project_dir: &str) -> Result<()> {
    let output = tokio::process::Command::new("npm")
        .args(["install"])
        .current_dir(project_dir)
        .output()
        .await
        .context(
            "Could not find 'npm'. Install Node.js: https://nodejs.org"
        )?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("npm install failed:\n{}", stderr);
    }
    Ok(())
}
