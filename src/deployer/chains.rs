use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhevmChain {
    pub name:                                     String,
    pub chain_id:                                 u64,
    pub gateway_chain_id:                         u64,
    pub rpc_env_var:                              String,
    pub acl_address:                              String,
    pub kms_verifier:                             String,
    pub input_verifier:                           String,
    pub verifying_contract_decryption:            String,
    pub verifying_contract_input_verification:    String,
    pub explorer_url:                             String,
    pub explorer_api_url:                         String,
    pub explorer_api_key_env:                     String,
}

impl FhevmChain {
    pub fn is_fully_configured(&self) -> bool {
        !self.acl_address.is_empty()
            && !self.kms_verifier.is_empty()
            && !self.input_verifier.is_empty()
    }
}

pub fn supported_chains() -> HashMap<&'static str, FhevmChain> {
    let mut m = HashMap::new();

    m.insert("sepolia", FhevmChain {
        name:                                  "Ethereum Sepolia".into(),
        chain_id:                              11155111,
        gateway_chain_id:                      55815,
        rpc_env_var:                           "SEPOLIA_RPC_URL".into(),
        acl_address:                           "0x687820221192C5B662b25367F70076A37bc79b6c".into(),
        kms_verifier:                          "0x1364cBBf2cDF5032C47d8226a6f6FBD2AFCDacAC".into(),
        input_verifier:                        "0xbc91f3daD1A5F19F8390c400196e58073B6a0BC4".into(),
        verifying_contract_decryption:         "0xb6E160B1ff80D67Bfe90A85eE06Ce0A2613607D1".into(),
        verifying_contract_input_verification: "0x7048C39f048125eDa9d678AEbaDfB22F7900a29F".into(),
        explorer_url:                          "https://sepolia.etherscan.io".into(),
        explorer_api_url:                      "https://api-sepolia.etherscan.io/api".into(),
        explorer_api_key_env:                  "ETHERSCAN_API_KEY".into(),
    });

    m.insert("mainnet", FhevmChain {
        name:                                  "Ethereum Mainnet".into(),
        chain_id:                              1,
        gateway_chain_id:                      55815,
        rpc_env_var:                           "MAINNET_RPC_URL".into(),
        acl_address:                           String::new(),
        kms_verifier:                          String::new(),
        input_verifier:                        String::new(),
        verifying_contract_decryption:         String::new(),
        verifying_contract_input_verification: String::new(),
        explorer_url:                          "https://etherscan.io".into(),
        explorer_api_url:                      "https://api.etherscan.io/api".into(),
        explorer_api_key_env:                  "ETHERSCAN_API_KEY".into(),
    });

    m.insert("base", FhevmChain {
        name:                                  "Base".into(),
        chain_id:                              8453,
        gateway_chain_id:                      55815,
        rpc_env_var:                           "BASE_RPC_URL".into(),
        acl_address:                           String::new(),
        kms_verifier:                          String::new(),
        input_verifier:                        String::new(),
        verifying_contract_decryption:         String::new(),
        verifying_contract_input_verification: String::new(),
        explorer_url:                          "https://basescan.org".into(),
        explorer_api_url:                      "https://api.basescan.org/api".into(),
        explorer_api_key_env:                  "BASESCAN_API_KEY".into(),
    });

    m.insert("arbitrum", FhevmChain {
        name:                                  "Arbitrum One".into(),
        chain_id:                              42161,
        gateway_chain_id:                      55815,
        rpc_env_var:                           "ARBITRUM_RPC_URL".into(),
        acl_address:                           String::new(),
        kms_verifier:                          String::new(),
        input_verifier:                        String::new(),
        verifying_contract_decryption:         String::new(),
        verifying_contract_input_verification: String::new(),
        explorer_url:                          "https://arbiscan.io".into(),
        explorer_api_url:                      "https://api.arbiscan.io/api".into(),
        explorer_api_key_env:                  "ARBISCAN_API_KEY".into(),
    });

    m
}
