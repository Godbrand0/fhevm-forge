use anyhow::{bail, Result};
use colored::Colorize;
use serde_json::{json, Value};
use std::{collections::HashMap, env, fs};
use crate::deployer::chains::supported_chains;

#[derive(Debug)]
pub struct DeployResult {
    pub chain:            String,
    pub contract_address: String,
    pub tx_hash:          String,
}

pub async fn run(chains: &[&str], contract: &str, dry_run: bool) -> Result<()> {
    let all_chains = supported_chains();
    let mut results: Vec<DeployResult> = Vec::new();
    let mut manifest: HashMap<String, Value> = HashMap::new();

    for &chain_key in chains {
        let chain = all_chains.get(chain_key)
            .ok_or_else(|| anyhow::anyhow!(
                "Unknown chain '{}'. Supported: {}",
                chain_key,
                all_chains.keys().cloned().collect::<Vec<_>>().join(", ")
            ))?;

        if !chain.is_fully_configured() {
            eprintln!(
                "{} Chain '{}' does not have FHEVM contract addresses configured yet. \
                 Check back when Zama deploys to this network.",
                "⚠️ ".yellow(),
                chain_key
            );
            continue;
        }

        let rpc_url = env::var(&chain.rpc_env_var).unwrap_or_default();
        if rpc_url.is_empty() {
            bail!(
                "Environment variable '{}' is not set. \
                 Add it to your .env file before deploying to {}.",
                chain.rpc_env_var,
                chain.name
            );
        }

        println!(
            "\n{} {} to {}...",
            "Deploying".cyan().bold(),
            contract.yellow(),
            chain.name.cyan()
        );

        let result = deploy_to_chain(chain_key, chain, contract, &rpc_url, dry_run).await;

        match result {
            Ok(r) => {
                println!(
                    "  {} {}  {}",
                    "✅".green(),
                    chain.name.green(),
                    r.contract_address.cyan()
                );
                println!(
                    "     {} {}/address/{}",
                    "Explorer:".dimmed(),
                    chain.explorer_url,
                    r.contract_address
                );

                manifest.insert(chain_key.to_string(), json!({
                    "address": r.contract_address,
                    "tx": r.tx_hash,
                    "chain_id": chain.chain_id,
                    "explorer": format!("{}/address/{}", chain.explorer_url, r.contract_address),
                }));

                results.push(r);
            }
            Err(e) => {
                eprintln!("  {} {} failed: {}", "❌".red(), chain.name, e);
            }
        }
    }

    if !results.is_empty() {
        fs::create_dir_all("deployments")?;
        let manifest_path = format!("deployments/{}.json", contract);
        let manifest_json = serde_json::to_string_pretty(&json!({
            "contract": contract,
            "deployed_at": chrono::Utc::now().to_rfc3339(),
            "deployments": manifest,
        }))?;
        fs::write(&manifest_path, manifest_json)?;

        println!("\n{} {}", "📋 Deployment manifest:".dimmed(), manifest_path.cyan());
    }

    Ok(())
}

async fn deploy_to_chain(
    chain_key: &str,
    chain: &crate::deployer::chains::FhevmChain,
    contract: &str,
    rpc_url: &str,
    dry_run: bool,
) -> Result<DeployResult> {
    let script_path = format!("script/Deploy{}.s.sol:Deploy{}Script", contract, contract);

    let mut cmd = tokio::process::Command::new("forge");
    cmd.args(["script", &script_path, "--rpc-url", rpc_url]);

    if !dry_run {
        cmd.args(["--broadcast", "--verify"]);
        let api_key = env::var(&chain.explorer_api_key_env).unwrap_or_default();
        if !api_key.is_empty() {
            cmd.args(["--etherscan-api-key", &api_key]);
        }
    }

    cmd.args(["-vvv"]);

    cmd.env("FHEVM_ACL_ADDRESS",                        &chain.acl_address)
       .env("FHEVM_KMS_VERIFIER",                       &chain.kms_verifier)
       .env("FHEVM_INPUT_VERIFIER",                     &chain.input_verifier)
       .env("FHEVM_VERIFYING_CONTRACT_DECRYPTION",      &chain.verifying_contract_decryption)
       .env("FHEVM_VERIFYING_CONTRACT_INPUT_VERIF",     &chain.verifying_contract_input_verification)
       .env("FHEVM_GATEWAY_CHAIN_ID",                   chain.gateway_chain_id.to_string())
       .env("DEPLOYER_PRIVATE_KEY",                     env::var("DEPLOYER_PRIVATE_KEY").unwrap_or_default());

    let output = cmd.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("forge script failed:\n{}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let address = parse_forge_address(&stdout)
        .unwrap_or_else(|| "unknown (check deployments/broadcast/)".to_string());
    let tx_hash = parse_forge_tx_hash(&stdout)
        .unwrap_or_else(|| "unknown".to_string());

    Ok(DeployResult {
        chain: chain_key.to_string(),
        contract_address: address,
        tx_hash,
    })
}

fn parse_forge_address(output: &str) -> Option<String> {
    output.lines()
        .find(|l| l.contains("Contract Address:"))
        .and_then(|l| l.split_whitespace().last())
        .map(|s| s.to_string())
}

fn parse_forge_tx_hash(output: &str) -> Option<String> {
    output.lines()
        .find(|l| l.contains("Transaction hash:"))
        .and_then(|l| l.split_whitespace().last())
        .map(|s| s.to_string())
}
