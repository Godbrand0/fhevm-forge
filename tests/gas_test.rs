use fhevm_forge::gas::costs::fhe_cost_table;
use fhevm_forge::gas::parser::count_fhe_ops;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cost_table_has_all_operations() {
    let costs = fhe_cost_table();

    let expected_ops = [
        "TFHE.add", "TFHE.sub", "TFHE.mul", "TFHE.div",
        "TFHE.lt",  "TFHE.le",  "TFHE.gt",  "TFHE.ge",  "TFHE.eq",
        "TFHE.select", "TFHE.and", "TFHE.or", "TFHE.not",
        "TFHE.asEuint64", "TFHE.asEuint128",
        "TFHE.allow", "TFHE.allowThis",
        "Gateway.requestDecryption",
    ];

    for op in &expected_ops {
        assert!(costs.contains_key(op), "Missing cost entry for: {}", op);
    }
}

#[test]
fn test_cost_table_mul_more_expensive_than_add() {
    let costs = fhe_cost_table();
    let (add_on, add_cop) = costs["TFHE.add"];
    let (mul_on, mul_cop) = costs["TFHE.mul"];
    assert!(mul_on  > add_on,  "TFHE.mul should cost more on-chain than TFHE.add");
    assert!(mul_cop > add_cop, "TFHE.mul should cost more coprocessor gas than TFHE.add");
}

#[test]
fn test_cost_table_div_most_expensive() {
    let costs = fhe_cost_table();
    let (_, div_cop) = costs["TFHE.div"];
    let (_, mul_cop) = costs["TFHE.mul"];
    assert!(div_cop > mul_cop, "TFHE.div should be more expensive than TFHE.mul");
}

#[test]
fn test_cost_table_allow_has_zero_coprocessor_cost() {
    let costs = fhe_cost_table();
    let (_, allow_cop)     = costs["TFHE.allow"];
    let (_, allow_this_cop) = costs["TFHE.allowThis"];
    assert_eq!(allow_cop,      0, "TFHE.allow should have no coprocessor cost");
    assert_eq!(allow_this_cop, 0, "TFHE.allowThis should have no coprocessor cost");
}

#[test]
fn test_parser_counts_tfhe_ops() {
    let tmp = TempDir::new().unwrap();
    let sol = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;
contract Test {
    euint64 a;
    euint64 b;
    function compute() public {
        euint64 sum  = TFHE.add(a, b);
        TFHE.allowThis(sum);
        euint64 prod = TFHE.mul(a, b);
        TFHE.allowThis(prod);
        euint64 prod2 = TFHE.mul(a, sum);
        TFHE.allowThis(prod2);
        TFHE.allow(sum, msg.sender);
    }
}
"#;
    fs::write(tmp.path().join("Test.sol"), sol).unwrap();

    let counts = count_fhe_ops(tmp.path().to_str().unwrap()).unwrap();
    assert_eq!(counts.get("TFHE.add").copied().unwrap_or(0),       1, "Expected 1 TFHE.add");
    assert_eq!(counts.get("TFHE.mul").copied().unwrap_or(0),       2, "Expected 2 TFHE.mul");
    assert_eq!(counts.get("TFHE.allowThis").copied().unwrap_or(0), 3, "Expected 3 TFHE.allowThis");
    assert_eq!(counts.get("TFHE.allow").copied().unwrap_or(0),     1, "Expected 1 TFHE.allow");
}

#[test]
fn test_parser_returns_empty_for_nonexistent_path() {
    let counts = count_fhe_ops("/nonexistent/path/xyz").unwrap();
    assert!(counts.is_empty(), "Should return empty map for nonexistent path");
}

#[test]
fn test_parser_ignores_non_sol_files() {
    let tmp = TempDir::new().unwrap();
    // Write a TypeScript file with TFHE refs — should NOT be counted
    fs::write(
        tmp.path().join("test.ts"),
        "const r = TFHE.add(a, b); TFHE.allowThis(r);"
    ).unwrap();

    let counts = count_fhe_ops(tmp.path().to_str().unwrap()).unwrap();
    // Parser only scans .sol files
    assert!(counts.is_empty(), "Should not count ops in .ts files");
}

#[test]
fn test_format_gas_correctness() {
    // Verify the cost table values match the spec
    let costs = fhe_cost_table();

    let (add_on, add_cop) = costs["TFHE.add"];
    assert_eq!(add_on,  8_000);
    assert_eq!(add_cop, 65_000);

    let (mul_on, mul_cop) = costs["TFHE.mul"];
    assert_eq!(mul_on,  15_000);
    assert_eq!(mul_cop, 150_000);

    let (gw_on, gw_cop) = costs["Gateway.requestDecryption"];
    assert_eq!(gw_on,  25_000);
    assert_eq!(gw_cop, 200_000);
}
