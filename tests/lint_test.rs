use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn lint_dir(src: &str) -> assert_cmd::assert::Assert {
    let tmp = TempDir::new().unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("Test.sol"), src).unwrap();

    Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["lint", src_dir.to_str().unwrap()])
        .output()
        .map(|_| ())
        .ok();

    Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["lint", src_dir.to_str().unwrap()])
        .assert()
}

fn lint_ts_dir(src: &str, filename: &str) -> assert_cmd::assert::Assert {
    let tmp = TempDir::new().unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join(filename), src).unwrap();

    Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["lint", src_dir.to_str().unwrap()])
        .assert()
}

#[test]
fn test_lint_catches_missing_allow_this() {
    let bad_sol = r#"
pragma solidity ^0.8.24;
import "fhevm/lib/TFHE.sol";
contract Bad {
    euint64 private value;
    function set(einput v, bytes calldata p) public {
        euint64 newVal = TFHE.asEuint64(v, p);
        value = newVal;
        // Missing TFHE.allowThis(newVal) — FHEVM-001
    }
}
"#;

    let tmp = TempDir::new().unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("Bad.sol"), bad_sol).unwrap();

    let output = Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["lint", src_dir.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("FHEVM-001"), "Expected FHEVM-001 in output:\n{}", stdout);
    assert!(!output.status.success(), "Should exit non-zero when errors found");
}

#[test]
fn test_lint_no_errors_for_correct_allow_this() {
    let good_sol = r#"
pragma solidity ^0.8.24;
import "fhevm/lib/TFHE.sol";
contract Good {
    euint64 private value;
    function set(einput v, bytes calldata p) public {
        euint64 newVal = TFHE.asEuint64(v, p);
        TFHE.allowThis(newVal);
        value = newVal;
    }
}
"#;

    let tmp = TempDir::new().unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("Good.sol"), good_sol).unwrap();

    let output = Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["lint", src_dir.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("FHEVM-001"), "Should not report FHEVM-001 for correct code:\n{}", stdout);
}

#[test]
fn test_lint_catches_view_fhe_function() {
    let bad_sol = r#"
pragma solidity ^0.8.24;
import "fhevm/lib/TFHE.sol";
contract Bad {
    euint64 private a;
    euint64 private b;
    function compute() public view returns (euint64) {
        return TFHE.add(a, b);
    }
}
"#;

    let tmp = TempDir::new().unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("Bad.sol"), bad_sol).unwrap();

    let output = Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["lint", src_dir.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("FHEVM-003"), "Expected FHEVM-003:\n{}", stdout);
}

#[test]
fn test_lint_catches_get_relayer_call() {
    let bad_ts = r#"
import { getFhevmInstance } from './fhevm';
async function decrypt(handles: bigint[]) {
    const fhe = await getFhevmInstance();
    // This is wrong — getRelayer() doesn't exist
    const result = await fhe.getRelayer().publicDecrypt(handles);
}
"#;

    let tmp = TempDir::new().unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("bad.ts"), bad_ts).unwrap();

    let output = Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["lint", src_dir.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("FHEVM-007"), "Expected FHEVM-007:\n{}", stdout);
}

#[test]
fn test_lint_catches_handle_index_oob() {
    let bad_ts = r#"
async function getHealth(vault: any, borrower: string) {
    const handles = await vault.getPositionHandles(borrower);
    const healthHandle = handles[2]; // FHEVM-008: only 2 values returned
}
"#;

    let tmp = TempDir::new().unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("bad.ts"), bad_ts).unwrap();

    let output = Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["lint", src_dir.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("FHEVM-008"), "Expected FHEVM-008:\n{}", stdout);
}

#[test]
fn test_lint_clean_path_exits_zero() {
    let tmp = TempDir::new().unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    // Write a minimal valid Solidity file with no FHEVM issues
    fs::write(src_dir.join("Clean.sol"), "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.27;\ncontract Clean {}").unwrap();

    Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["lint", src_dir.to_str().unwrap()])
        .assert()
        .success();
}
