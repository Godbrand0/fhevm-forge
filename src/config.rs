use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct FhevmForgeConfig {
    #[serde(default)]
    pub deploy: DeployConfig,

    #[serde(default)]
    pub lint: LintConfig,

    #[serde(default)]
    pub gas: GasConfig,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct DeployConfig {
    #[serde(default)]
    pub chains: Vec<String>,
    pub default_contract: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct LintConfig {
    #[serde(default)]
    pub ignore: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct GasConfig {
    pub output: Option<Vec<String>>,
    pub json_path: Option<String>,
    pub warn_if_coprocessor_gas_exceeds: Option<u64>,
}

impl FhevmForgeConfig {
    pub fn load() -> Result<Self> {
        let path = "fhevm-forge.toml";
        if !std::path::Path::new(path).exists() {
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(path)?;
        let config: FhevmForgeConfig = toml::from_str(&contents)?;
        Ok(config)
    }
}
