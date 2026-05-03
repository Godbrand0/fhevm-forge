use anyhow::{Context, Result, bail};
use colored::Colorize;
use dialoguer::{Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use std::{fs, path::Path};
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

    // Write all local files immediately — no network, instant.
    pb.set_message("Generating contract and SDK files...");
    let generator = Generator::new(name, &template)?;
    generator.render_all().context("Template rendering failed")?;
    generator.write_config_files().context("Failed to write config files")?;

    // forge install → patch → soldeer install  run concurrently with  npm install.
    // The two chains share no files, so they're fully independent.
    pb.set_message("Installing dependencies (forge + npm in parallel)...");
    let name_clone = name.to_string();
    let forge_chain = async {
        forge_install(&name_clone, "zama-ai/forge-fhevm").await
            .context("forge install zama-ai/forge-fhevm failed")?;
        // Remove the @openzeppelin-confidential-contracts git dep from forge-fhevm's
        // foundry.toml before soldeer runs. That package is a full repo clone (~198 MB)
        // used only for two interface files, which we stub locally instead.
        patch_forge_fhevm_foundry_toml(&name_clone)
            .context("failed to patch lib/forge-fhevm/foundry.toml")?;
        create_oz_confidential_stubs(&name_clone)
            .context("failed to write OZ confidential interface stubs")?;
        soldeer_install(&name_clone).await
            .context("forge soldeer install (inside lib/forge-fhevm) failed")
    };
    tokio::try_join!(forge_chain, npm_install(name))?;

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

/// Remove the @openzeppelin-confidential-contracts git dependency from forge-fhevm's
/// foundry.toml so soldeer skips the ~198 MB full-repo clone. We replace it with
/// minimal interface stubs written by `create_oz_confidential_stubs`.
fn patch_forge_fhevm_foundry_toml(project_dir: &str) -> Result<()> {
    let toml_path = Path::new(project_dir).join("lib/forge-fhevm/foundry.toml");
    let content = fs::read_to_string(&toml_path)
        .context("could not read lib/forge-fhevm/foundry.toml")?;
    let patched: String = content
        .lines()
        .filter(|line| !line.contains("@openzeppelin-confidential-contracts"))
        .collect::<Vec<_>>()
        .join("\n");
    let patched = if content.ends_with('\n') { patched + "\n" } else { patched };
    fs::write(&toml_path, patched)
        .context("could not write patched lib/forge-fhevm/foundry.toml")?;
    Ok(())
}

/// Write minimal stubs for the two interfaces that forge-fhevm imports from
/// @openzeppelin-confidential-contracts. Placed at the exact soldeer path so
/// forge-fhevm's existing remappings resolve them without any project-level changes.
fn create_oz_confidential_stubs(project_dir: &str) -> Result<()> {
    let stub_dir = Path::new(project_dir)
        .join("lib/forge-fhevm/dependencies/@openzeppelin-confidential-contracts-6edd293/contracts/interfaces");
    fs::create_dir_all(&stub_dir)
        .context("could not create OZ confidential stub directory")?;
    fs::write(stub_dir.join("IERC7984.sol"), IERC7984_STUB)
        .context("could not write IERC7984.sol stub")?;
    fs::write(stub_dir.join("IERC7984ERC20Wrapper.sol"), IERC7984_WRAPPER_STUB)
        .context("could not write IERC7984ERC20Wrapper.sol stub")?;
    Ok(())
}

// ── Interface stubs ────────────────────────────────────────────────────────────
// These replace the full @openzeppelin-confidential-contracts repo clone.
// The IERC165 import uses the @openzeppelin-contracts/ remapping that soldeer
// sets up from forge-fhevm's own foundry.toml — no extra remappings needed.

const IERC7984_STUB: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {euint64, externalEuint64} from "@fhevm/solidity/lib/FHE.sol";
import {IERC165} from "@openzeppelin-contracts/contracts/interfaces/IERC165.sol";

interface IERC7984 is IERC165 {
    event OperatorSet(address indexed holder, address indexed operator, uint48 until);
    event ConfidentialTransfer(address indexed from, address indexed to, euint64 indexed amount);
    event AmountDisclosed(euint64 indexed encryptedAmount, uint64 amount);

    function name() external view returns (string memory);
    function symbol() external view returns (string memory);
    function decimals() external view returns (uint8);
    function contractURI() external view returns (string memory);
    function confidentialTotalSupply() external view returns (euint64);
    function confidentialBalanceOf(address account) external view returns (euint64);
    function isOperator(address holder, address spender) external view returns (bool);
    function setOperator(address operator, uint48 until) external;

    function confidentialTransfer(address to, externalEuint64 encryptedAmount, bytes calldata inputProof) external returns (euint64);
    function confidentialTransfer(address to, euint64 amount) external returns (euint64 transferred);
    function confidentialTransferFrom(address from, address to, externalEuint64 encryptedAmount, bytes calldata inputProof) external returns (euint64);
    function confidentialTransferFrom(address from, address to, euint64 amount) external returns (euint64 transferred);
    function confidentialTransferAndCall(address to, externalEuint64 encryptedAmount, bytes calldata inputProof, bytes calldata data) external returns (euint64 transferred);
    function confidentialTransferAndCall(address to, euint64 amount, bytes calldata data) external returns (euint64 transferred);
    function confidentialTransferFromAndCall(address from, address to, externalEuint64 encryptedAmount, bytes calldata inputProof, bytes calldata data) external returns (euint64 transferred);
    function confidentialTransferFromAndCall(address from, address to, euint64 amount, bytes calldata data) external returns (euint64 transferred);
}
"#;

const IERC7984_WRAPPER_STUB: &str = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {externalEuint64, euint64} from "@fhevm/solidity/lib/FHE.sol";
import {IERC7984} from "./IERC7984.sol";

interface IERC7984ERC20Wrapper is IERC7984 {
    function wrap(address to, uint256 amount) external;
    function unwrap(address from, address to, externalEuint64 encryptedAmount, bytes calldata inputProof) external returns (euint64);
    function underlying() external view returns (address);
}
"#;

// ── Forge / npm helpers ────────────────────────────────────────────────────────

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
        .args(["install", "--no-audit", "--no-fund"])
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
