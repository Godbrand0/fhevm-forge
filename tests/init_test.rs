use assert_cmd::Command;
use tempfile::TempDir;

fn forge_available() -> bool {
    std::process::Command::new("forge")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn test_init_lending_template() {
    if !forge_available() { return; }
    let tmp = TempDir::new().unwrap();
    let project_name = "test-lending";

    Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["init", project_name, "--template", "lending"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let project_dir = tmp.path().join(project_name);
    assert!(project_dir.join("src/ConfidentialVault.sol").exists());
    assert!(project_dir.join("lib/fhevm/instance.ts").exists());
    assert!(project_dir.join("lib/fhevm/decrypt.ts").exists());
    assert!(project_dir.join("agent/lib/fhevm-agent.ts").exists());
    assert!(project_dir.join("AGENT.md").exists());
    assert!(project_dir.join("foundry.toml").exists());
    assert!(project_dir.join("fhevm-forge.toml").exists());
    assert!(project_dir.join(".env.example").exists());
}

#[test]
fn test_init_blank_template() {
    if !forge_available() { return; }
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["init", "test-blank", "--template", "blank"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let project_dir = tmp.path().join("test-blank");
    assert!(project_dir.join("src/Counter.sol").exists());
    assert!(project_dir.join("test/Counter.t.sol").exists());
    assert!(project_dir.join("AGENT.md").exists());
}

#[test]
fn test_init_erc7984_template() {
    if !forge_available() { return; }
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["init", "test-erc7984", "--template", "erc7984"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let project_dir = tmp.path().join("test-erc7984");
    assert!(project_dir.join("src/ConfidentialToken.sol").exists());
    assert!(project_dir.join("script/Deploy.s.sol").exists());
}

#[test]
fn test_init_auction_template() {
    if !forge_available() { return; }
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["init", "test-auction", "--template", "auction"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let project_dir = tmp.path().join("test-auction");
    assert!(project_dir.join("src/BlindAuction.sol").exists());
    assert!(project_dir.join("test/BlindAuction.t.sol").exists());
}

#[test]
fn test_init_voting_template() {
    if !forge_available() { return; }
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["init", "test-voting", "--template", "voting"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let project_dir = tmp.path().join("test-voting");
    assert!(project_dir.join("src/ConfidentialVoting.sol").exists());
    assert!(project_dir.join("test/ConfidentialVoting.t.sol").exists());
}

#[test]
fn test_init_rejects_existing_dir() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir(tmp.path().join("existing")).unwrap();

    Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["init", "existing", "--template", "blank"])
        .current_dir(tmp.path())
        .assert()
        .failure();
}

#[test]
fn test_init_rejects_unknown_template() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["init", "test-proj", "--template", "nonexistent"])
        .current_dir(tmp.path())
        .assert()
        .failure();
}

#[test]
fn test_init_writes_foundry_toml_with_cancun() {
    if !forge_available() { return; }
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["init", "test-foundry", "--template", "blank"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let foundry_toml = std::fs::read_to_string(
        tmp.path().join("test-foundry").join("foundry.toml")
    ).unwrap();
    assert!(foundry_toml.contains("cancun"), "foundry.toml must have evm_version = \"cancun\"");
    assert!(foundry_toml.contains("0.8.27"),  "foundry.toml must specify solc version");
}
