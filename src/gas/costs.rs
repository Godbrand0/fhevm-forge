// FHE operation cost table.
// All costs are estimates. On-chain gas is from EVM symbolic execution.
// Coprocessor gas is the off-chain compute cost billed through the Zama protocol.

use std::collections::HashMap;

/// Returns a map of FHE operation name → (on_chain_gas, coprocessor_gas).
pub fn fhe_cost_table() -> HashMap<&'static str, (u64, u64)> {
    let mut m = HashMap::new();
    m.insert("TFHE.add",                  (8_000,  65_000));
    m.insert("TFHE.sub",                  (8_000,  65_000));
    m.insert("TFHE.mul",                  (15_000, 150_000));
    m.insert("TFHE.div",                  (30_000, 400_000));
    m.insert("TFHE.lt",                   (10_000, 70_000));
    m.insert("TFHE.le",                   (10_000, 70_000));
    m.insert("TFHE.gt",                   (10_000, 70_000));
    m.insert("TFHE.ge",                   (10_000, 70_000));
    m.insert("TFHE.eq",                   (10_000, 70_000));
    m.insert("TFHE.select",               (12_000, 90_000));
    m.insert("TFHE.and",                  (5_000,  30_000));
    m.insert("TFHE.or",                   (5_000,  30_000));
    m.insert("TFHE.not",                  (5_000,  30_000));
    m.insert("TFHE.asEuint64",            (6_000,  50_000));
    m.insert("TFHE.asEuint128",           (6_500,  55_000));
    m.insert("TFHE.allow",                (3_000,  0));
    m.insert("TFHE.allowThis",            (3_000,  0));
    m.insert("Gateway.requestDecryption", (25_000, 200_000));
    m
}
