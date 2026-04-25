# fhevm-forge — Master Specification
> Read this file first. It defines what the project is, the full repository
> structure, the Cargo.toml, and the precise build/test/publish workflow.
> After reading this, read IMPLEMENTATION.md, TEMPLATES.md, and AGENT_MD_TEMPLATE.md.

---

## What This Project Is

`fhevm-forge` is a Rust CLI tool that serves as the primary developer experience
layer for building confidential smart contracts on Zama FHEVM using Foundry.

It does four things:

1. **Scaffold** (`fhevm-forge init`) — Generates a complete Foundry project wired
   to `forge-fhevm` and `@zama-fhe/relayer-sdk`, with Solidity contracts,
   TypeScript SDK integration, React hooks, agent runtime, and documentation.

2. **Deploy** (`fhevm-forge deploy`) — Runs `forge script` against one or more
   chains simultaneously, injecting chain-specific FHEVM contract addresses and
   producing a deployment manifest JSON.

3. **Gas** (`fhevm-forge gas`) — Parses `forge test --gas-report` output and
   overlays FHE-specific coprocessor cost estimates on top of standard EVM gas,
   producing a report showing the true cost of each FHE operation.

4. **Lint** (`fhevm-forge lint`) — Runs a Solidity static analyzer that checks
   for FHE-specific bugs that no general-purpose linter catches: missing
   `TFHE.allowThis()`, wrong function visibility on FHE ops, missing `onlyGateway`
   modifiers, Relayer SDK misuse, and more.

---

## Why Rust

- **Single binary distribution** — `cargo install fhevm-forge` or download a
  prebuilt binary. No Node.js, no npm, no version conflicts.
- **Foundry alignment** — Foundry itself is Rust. A Rust CLI fits the toolchain.
- **Template embedding** — `include_str!()` embeds all template files at compile
  time. The binary is fully self-contained.
- **Author familiarity** — Author is already building an EVM in Rust.

---

## Repository Layout

```
fhevm-forge/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── .github/
│   └── workflows/
│       ├── ci.yml              # cargo test + cargo clippy on every PR
│       └── release.yml         # Cross-compile + publish to crates.io on tag
├── src/
│   ├── main.rs                 # CLI entry point — clap commands
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── init.rs             # fhevm-forge init
│   │   ├── deploy.rs           # fhevm-forge deploy
│   │   ├── gas.rs              # fhevm-forge gas
│   │   ├── lint.rs             # fhevm-forge lint
│   │   └── doctor.rs           # fhevm-forge doctor
│   ├── scaffold/
│   │   ├── mod.rs
│   │   ├── generator.rs        # Template rendering + file writing
│   │   └── templates.rs        # Template registry (include_str! table)
│   ├── deployer/
│   │   ├── mod.rs
│   │   ├── chains.rs           # Chain registry structs + data
│   │   └── runner.rs           # forge script subprocess executor
│   ├── gas/
│   │   ├── mod.rs
│   │   ├── costs.rs            # FHE op cost table
│   │   ├── parser.rs           # Parse forge gas-report output
│   │   └── reporter.rs         # Terminal + JSON + Markdown output
│   ├── linter/
│   │   ├── mod.rs              # Linter orchestrator
│   │   ├── reporter.rs         # Error display
│   │   └── rules/
│   │       ├── mod.rs          # LintRule trait + LintError struct
│   │       ├── missing_allow_this.rs
│   │       ├── missing_allow_addr.rs
│   │       ├── view_fhe_function.rs
│   │       ├── euint_overflow.rs
│   │       ├── input_proof_reuse.rs
│   │       ├── get_relayer_call.rs
│   │       ├── missing_only_gateway.rs
│   │       ├── handle_index_oob.rs
│   │       └── resolver_arg_count.rs
│   └── config.rs               # fhevm-forge.toml loader
├── templates/
│   ├── shared/
│   │   ├── AGENT.md            # AI coding agent instruction file
│   │   ├── README.md.tera      # README template (rendered with project name)
│   │   ├── foundry.toml.tera
│   │   ├── fhevm-forge.toml
│   │   ├── .env.example
│   │   └── lib/
│   │       └── fhevm/
│   │           ├── instance.ts
│   │           ├── encrypt.ts
│   │           ├── decrypt.ts
│   │           ├── gateway.ts
│   │           ├── errors.ts
│   │           ├── config.ts
│   │           └── index.ts
│   ├── shared/hooks/
│   │   ├── useEncrypt.ts
│   │   ├── useReencrypt.ts
│   │   └── useHealthCheck.ts
│   ├── shared/agent/
│   │   └── fhevm-agent.ts
│   ├── blank/
│   │   ├── src/Counter.sol
│   │   └── test/Counter.t.sol
│   ├── erc7984/
│   │   ├── src/ConfidentialToken.sol
│   │   ├── test/ConfidentialToken.t.sol
│   │   └── script/Deploy.s.sol
│   ├── lending/
│   │   ├── src/
│   │   │   ├── ConfidentialVault.sol
│   │   │   ├── tokens/ConfidentialCollateral.sol
│   │   │   └── tokens/ConfidentialDebt.sol
│   │   ├── test/ConfidentialVault.t.sol
│   │   └── script/Deploy.s.sol
│   ├── auction/
│   │   ├── src/BlindAuction.sol
│   │   ├── test/BlindAuction.t.sol
│   │   └── script/Deploy.s.sol
│   └── voting/
│       ├── src/ConfidentialVoting.sol
│       ├── test/ConfidentialVoting.t.sol
│       └── script/Deploy.s.sol
└── tests/
    ├── init_test.rs            # Integration test: scaffold a project end-to-end
    ├── lint_test.rs            # Integration test: lint known-bad Solidity files
    └── gas_test.rs             # Unit test: FHE cost table + parser
```

---

## Cargo.toml

```toml
[package]
name        = "fhevm-forge"
version     = "0.1.0"
edition     = "2021"
description = "Foundry scaffold, deployer, gas estimator and linter for Zama FHEVM"
license     = "MIT"
repository  = "https://github.com/yourusername/fhevm-forge"
keywords    = ["fhevm", "zama", "foundry", "fhe", "blockchain"]
categories  = ["command-line-utilities", "cryptography"]

[[bin]]
name = "fhevm-forge"
path = "src/main.rs"

[dependencies]
# CLI
clap        = { version = "4",    features = ["derive", "env"] }

# Async runtime
tokio       = { version = "1",    features = ["full"] }

# Template rendering
tera        = "1"

# Terminal UI
colored     = "2"
indicatif   = "0.17"
dialoguer   = "0.11"

# File system
fs_extra    = "1"
walkdir     = "2"

# Serialization
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
toml        = "0.8"

# Error handling
anyhow      = "1"
thiserror   = "1"

# HTTP (doctor command)
reqwest     = { version = "0.12", features = ["json"] }

# Regex (linter pattern matching)
regex       = "1"

[dev-dependencies]
tempfile    = "3"
assert_cmd  = "2"
predicates  = "3"
```

---

## CLI Surface — All Commands

```
fhevm-forge 0.1.0
Foundry scaffold, deployer, gas estimator and linter for Zama FHEVM

USAGE:
    fhevm-forge <SUBCOMMAND>

SUBCOMMANDS:
    init      Scaffold a new FHEVM Foundry project
    deploy    Deploy contracts to one or more chains
    gas       Generate FHE gas cost report
    lint      Run FHEVM static analyzer on Solidity source
    doctor    Check development environment setup
    help      Print help information

fhevm-forge init <NAME> [OPTIONS]
    -t, --template <TEMPLATE>    blank | erc7984 | lending | auction | voting
                                 [default: interactive prompt]

fhevm-forge deploy [OPTIONS]
    -c, --chains <CHAINS>        Comma-separated: sepolia,mainnet,base,arbitrum
                                 [default: sepolia]
        --contract <CONTRACT>    Contract name to deploy [default: from config]
        --dry-run                Simulate without broadcasting

fhevm-forge gas [OPTIONS]
    -c, --contract <CONTRACT>    Contract to analyze [default: all]
    -o, --output <FORMAT>        terminal | json | markdown [default: terminal]

fhevm-forge lint [PATH] [OPTIONS]
    PATH                         Directory to analyze [default: ./src]
        --fix                    Auto-fix safe issues (FHEVM-001, FHEVM-003)

fhevm-forge doctor
    (no options — checks forge, node, env vars, forge-fhevm installation)
```

---

## Build, Test, Publish Workflow

```bash
# Development
cargo build                        # compile
cargo run -- init my-test --template lending  # test locally
cargo run -- lint ./fixtures/bad/  # test linter
cargo clippy -- -D warnings        # lint Rust code
cargo fmt                          # format

# Testing
cargo test                         # unit + integration tests

# Release build
cargo build --release
./target/release/fhevm-forge --version

# Publish to crates.io
cargo publish

# Cross-compilation (done by GitHub Actions on tag push)
# Targets: x86_64-unknown-linux-gnu
#          x86_64-apple-darwin
#          aarch64-apple-darwin (Apple Silicon)
#          x86_64-pc-windows-msvc
```

---

## GitHub Actions: CI

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test
      - run: cargo clippy -- -D warnings
      - run: cargo fmt --check
```

---

## GitHub Actions: Release

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags: ['v*']
jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: fhevm-forge-linux-x86_64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: fhevm-forge-macos-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: fhevm-forge-macos-arm64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: fhevm-forge-windows-x86_64.exe
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - name: Upload artifact
        uses: actions/upload-release-asset@v1
        with:
          asset_path: ./target/${{ matrix.target }}/release/fhevm-forge
          asset_name: ${{ matrix.artifact }}

  publish:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo publish --token ${{ secrets.CRATES_IO_TOKEN }}
```

---

## Key Design Decisions

### Templates Are Embedded at Compile Time
Use `include_str!()` for every template file. The binary ships with all
templates baked in. No external file reads at runtime. This means:
- `fhevm-forge init` works offline
- No path issues across operating systems
- Single binary deployment

### Tera for Template Rendering
Templates use Tera syntax (Jinja2-like). Variables available in every template:
- `{{ project_name }}` — the name passed to `init`
- `{{ template }}` — which template was chosen
- `{{ fhevm_version }}` — current relayer SDK version (hardcoded in source)
- `{{ year }}` — current year (for copyright headers)

For files that don't need variable substitution (e.g. TypeScript SDK files),
use `include_str!()` and copy as-is.

### Subprocess for Forge
All `forge` interactions use `tokio::process::Command`. Never shell out
with `std::process::Command` — always async to avoid blocking the runtime.
Always capture both stdout and stderr. On failure, print the full forge
error output so the developer can debug.

### Linter Uses Regex, Not Full AST
A full Solidity AST parser (`solar-parse`) adds significant compile time and
complexity. For the patterns we need to detect, regex over source lines is
sufficient and far simpler to maintain. Each lint rule gets a focused regex
that matches the pattern it's looking for. This is intentional — document it.

### Error Handling
Use `anyhow` for all application errors (propagation with `?`).
Use `thiserror` for domain error types (LintError, DeployError, etc.).
Never use `unwrap()` or `expect()` in non-test code except where truly
impossible to fail (e.g., compiling a regex at startup with `unwrap()`
on a literal pattern).

---

## Lint Rule IDs and Severities

| ID | Severity | Description |
|----|----------|-------------|
| FHEVM-001 | Error | `euint*` assigned but `TFHE.allowThis()` not called in scope |
| FHEVM-002 | Error | `euint*` handle passed to external contract without `TFHE.allow()` |
| FHEVM-003 | Error | Function marked `view` or `pure` but contains TFHE operation |
| FHEVM-004 | Warning | `euint64` used in `TFHE.mul()` — possible overflow for large values |
| FHEVM-005 | Error | Same `inputProof` variable passed to multiple `TFHE.asEuint*()` calls |
| FHEVM-006 | Error | Gateway callback function missing `onlyGateway` modifier |
| FHEVM-007 | Error | `.getRelayer()` called on an fhevm instance (method does not exist) |
| FHEVM-008 | Error | `handles[N]` accessed where N >= known getter return count |
| FHEVM-009 | Error | Resolver function called with fewer than 3 required arguments |

---

## FHE Gas Cost Table (used by `gas` command)

All costs are estimates. On-chain gas is from EVM symbolic execution.
Coprocessor gas is the off-chain compute cost billed through the Zama protocol.

| Operation | On-Chain Gas | Coprocessor Gas |
|-----------|-------------|-----------------|
| TFHE.add | 8,000 | 65,000 |
| TFHE.sub | 8,000 | 65,000 |
| TFHE.mul | 15,000 | 150,000 |
| TFHE.div | 30,000 | 400,000 |
| TFHE.lt | 10,000 | 70,000 |
| TFHE.le | 10,000 | 70,000 |
| TFHE.gt | 10,000 | 70,000 |
| TFHE.ge | 10,000 | 70,000 |
| TFHE.eq | 10,000 | 70,000 |
| TFHE.select | 12,000 | 90,000 |
| TFHE.and | 5,000 | 30,000 |
| TFHE.or | 5,000 | 30,000 |
| TFHE.not | 5,000 | 30,000 |
| TFHE.asEuint64 | 6,000 | 50,000 |
| TFHE.asEuint128 | 6,500 | 55,000 |
| TFHE.allow | 3,000 | 0 |
| TFHE.allowThis | 3,000 | 0 |
| Gateway.requestDecryption | 25,000 | 200,000 |

---

## Chain Registry — Sepolia Addresses (Confirmed)

These are Zama's deployed contract addresses on Sepolia testnet.
Use these exact values. Do not make up addresses.

```
chainId:                              11155111
gatewayChainId:                       55815
aclContractAddress:                   0x687820221192C5B662b25367F70076A37bc79b6c
kmsContractAddress:                   0x1364cBBf2cDF5032C47d8226a6f6FBD2AFCDacAC
inputVerifierContractAddress:         0xbc91f3daD1A5F19F8390c400196e58073B6a0BC4
verifyingContractAddressDecryption:   0xb6E160B1ff80D67Bfe90A85eE06Ce0A2613607D1
verifyingContractAddressInputVerif:   0x7048C39f048125eDa9d678AEbaDfB22F7900a29F
```

For mainnet, base, and arbitrum — use empty strings as placeholders.
The deploy command should error clearly if a user tries to deploy to a chain
with empty addresses.
