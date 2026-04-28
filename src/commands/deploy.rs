use anyhow::{bail, Result};
use colored::Colorize;
use serde_json::{json, Value};
use std::{collections::HashMap, env, fs};
use crate::deployer::{chains::supported_chains, runner::ForgeRunner};
use crate::config::FhevmForgeConfig;

#[derive(Debug)]
pub struct DeployResult {
    pub chain:            String,
    pub contract_address: String,
    pub tx_hash:          String,
}

pub async fn run(chains: &[&str], contract: &str, dry_run: bool) -> Result<()> {
    let cfg = FhevmForgeConfig::load()?;

    // Resolve chains: CLI arg takes precedence; fall back to config defaults.
    let default_chains: Vec<String> = if chains == ["sepolia"] && !cfg.deploy.chains.is_empty() {
        cfg.deploy.chains.clone()
    } else {
        chains.iter().map(|s| s.to_string()).collect()
    };
    let chains_to_deploy: Vec<&str> = default_chains.iter().map(String::as_str).collect();

    // Resolve contract: CLI arg takes precedence; fall back to config default.
    let resolved_contract = if contract.is_empty() {
        cfg.deploy.default_contract.as_deref().unwrap_or(contract)
    } else {
        contract
    };

    if resolved_contract.is_empty() {
        bail!("No contract specified. Pass --contract <Name> or set [deploy] default_contract in fhevm-forge.toml");
    }

    let all_chains = supported_chains();
    let (chains, contract) = (chains_to_deploy.as_slice(), resolved_contract);
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

        let deployed_chains: Vec<&str> = results.iter().map(|r| r.chain.as_str()).collect();
        println!(
            "\n{} {} deployed to: {}",
            "📋".dimmed(),
            contract.yellow(),
            deployed_chains.join(", ").cyan()
        );
        println!("{} {}", "Deployment manifest:".dimmed(), manifest_path.cyan());
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

    let mut runner = ForgeRunner::new(rpc_url).verbose(true);

    if !dry_run {
        runner = runner.arg("--verify");
        let api_key = env::var(&chain.explorer_api_key_env).unwrap_or_default();
        if !api_key.is_empty() {
            runner = runner.arg("--etherscan-api-key").arg(api_key);
        }
    }

    runner = runner
        .env("FHEVM_ACL_ADDRESS",                    &chain.acl_address)
        .env("FHEVM_KMS_VERIFIER",                   &chain.kms_verifier)
        .env("FHEVM_INPUT_VERIFIER",                 &chain.input_verifier)
        .env("FHEVM_VERIFYING_CONTRACT_DECRYPTION",  &chain.verifying_contract_decryption)
        .env("FHEVM_VERIFYING_CONTRACT_INPUT_VERIF", &chain.verifying_contract_input_verification)
        .env("FHEVM_GATEWAY_CHAIN_ID",               chain.gateway_chain_id.to_string())
        .env("DEPLOYER_PRIVATE_KEY",                 env::var("DEPLOYER_PRIVATE_KEY").unwrap_or_default());

    let stdout = runner.run_script(&script_path, !dry_run).await?;

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
