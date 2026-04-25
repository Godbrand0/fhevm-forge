# fhevm-forge — Implementation Guide
> This file contains the complete Rust implementation for every module.
> Read SPEC.md first for project context and repository layout.
> Implement modules in the order they appear in this file.

---

## Module 1: `src/main.rs`

Implement the CLI entry point using clap derive macros.
Every subcommand delegates to its command module immediately.
No business logic lives in main.rs.

```rust
use clap::{Parser, Subcommand};
use anyhow::Result;

mod commands;
mod scaffold;
mod deployer;
mod gas;
mod linter;
mod config;

#[derive(Parser)]
#[command(
    name    = "fhevm-forge",
    about   = "Foundry scaffold, deployer, gas estimator and linter for Zama FHEVM",
    version = env!("CARGO_PKG_VERSION"),
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scaffold a new FHEVM Foundry project
    Init {
        /// Project name and directory to create
        name: String,

        /// Starting template
        /// Options: blank | erc7984 | lending | auction | voting
        #[arg(short, long)]
        template: Option<String>,
    },

    /// Deploy contracts to one or more chains
    Deploy {
        /// Comma-separated chain keys: sepolia,mainnet,base,arbitrum
        #[arg(short, long, default_value = "sepolia")]
        chains: String,

        /// Contract name (matches Deploy<Name>.s.sol script)
        #[arg(short, long)]
        contract: String,

        /// Simulate without broadcasting transactions
        #[arg(long)]
        dry_run: bool,
    },

    /// Generate FHE-aware gas cost report
    Gas {
        /// Contract to analyze (default: all contracts)
        #[arg(short, long)]
        contract: Option<String>,

        /// Output format: terminal | json | markdown
        #[arg(short, long, default_value = "terminal")]
        output: String,
    },

    /// Run FHEVM static analyzer on Solidity source files
    Lint {
        /// Path to analyze
        #[arg(default_value = "./src")]
        path: String,

        /// Auto-fix safe issues (FHEVM-001, FHEVM-003)
        #[arg(long)]
        fix: bool,
    },

    /// Check FHEVM development environment
    Doctor,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name, template } => {
            commands::init::run(&name, template.as_deref()).await
        }
        Commands::Deploy { chains, contract, dry_run } => {
            let chain_list: Vec<&str> = chains.split(',').map(str::trim).collect();
            commands::deploy::run(&chain_list, &contract, dry_run).await
        }
        Commands::Gas { contract, output } => {
            commands::gas::run(contract.as_deref(), &output).await
        }
        Commands::Lint { path, fix } => {
            commands::lint::run(&path, fix).await
        }
        Commands::Doctor => {
            commands::doctor::run().await
        }
    }
}
```

---

## Module 2: `src/commands/init.rs`

The init command does five things in sequence:
1. Resolve template (prompt if not provided via flag)
2. Run `forge init <name> --no-git`
3. Run `forge install zama-ai/forge-fhevm --no-commit`
4. Render and copy all template files for the chosen template
5. Write shared config files (foundry.toml, fhevm-forge.toml, .env.example, AGENT.md, README.md)

```rust
use anyhow::{Context, Result, bail};
use colored::Colorize;
use dialoguer::{Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use crate::scaffold::generator::Generator;

// Template display names shown in the interactive prompt
const TEMPLATES: &[(&str, &str)] = &[
    ("blank",   "Blank FHEVM Project (bare Foundry + forge-fhevm)"),
    ("erc7984", "Confidential ERC-7984 Token"),
    ("lending", "Confidential Lending Vault (Vault + cETH + cUSDC)"),
    ("auction", "Blind Dutch Auction"),
    ("voting",  "Confidential Voting System"),
];

pub async fn run(name: &str, template_flag: Option<&str>) -> Result<()> {
    // Validate project name
    if name.is_empty() {
        bail!("Project name cannot be empty");
    }

    let target = Path::new(name);
    if target.exists() {
        bail!("Directory '{}' already exists. Choose a different name.", name);
    }

    // Resolve template
    let template = match template_flag {
        Some(t) => {
            // Validate the provided template name
            if !TEMPLATES.iter().any(|(k, _)| *k == t) {
                let valid: Vec<&str> = TEMPLATES.iter().map(|(k, _)| *k).collect();
                bail!("Unknown template '{}'. Valid options: {}", t, valid.join(", "));
            }
            t.to_string()
        }
        None => {
            // Interactive prompt
            let labels: Vec<&str> = TEMPLATES.iter().map(|(_, l)| *l).collect();
            let idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose a starting template")
                .items(&labels)
                .default(2) // default to lending (most complete example)
                .interact()
                .context("Failed to show template selector")?;
            TEMPLATES[idx].0.to_string()
        }
    };

    println!(
        "\n{} {} project in {}/\n",
        "Scaffolding".cyan().bold(),
        template.yellow(),
        name
    );

    // Progress spinner
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    // Step 1: forge init
    pb.set_message("Running forge init...");
    forge_init(name).await.context("forge init failed")?;

    // Step 2: Install forge-fhevm
    pb.set_message("Installing zama-ai/forge-fhevm...");
    forge_install(name, "zama-ai/forge-fhevm").await
        .context("forge install zama-ai/forge-fhevm failed")?;

    // Step 3: Render template files
    pb.set_message("Generating contract and SDK files...");
    let generator = Generator::new(name, &template)?;
    generator.render_all().context("Template rendering failed")?;

    // Step 4: Write config files
    pb.set_message("Writing configuration files...");
    generator.write_config_files().context("Failed to write config files")?;

    pb.finish_and_clear();

    // Success output
    println!("{}\n", "✅ Project scaffolded successfully!".green().bold());
    println!("  {} {}", "cd".dimmed(), name.cyan());
    println!("  {}  # Run FHE tests (uses forge-fhevm local mock)", "forge test".cyan());
    println!("  {}      # Estimate FHE gas costs", "fhevm-forge gas".cyan());
    println!("  {}     # Check for TFHE.allow() bugs", "fhevm-forge lint".cyan());
    println!("  {} --chains sepolia  # Deploy to testnet", "fhevm-forge deploy".cyan());
    println!("\n  Read {} for FHEVM development guidelines.", "AGENT.md".yellow());

    Ok(())
}

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

async fn forge_install(project_dir: &str, dep: &str) -> Result<()> {
    let output = tokio::process::Command::new("forge")
        .args(["install", dep, "--no-commit"])
        .current_dir(project_dir)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("forge install {} failed:\n{}", dep, stderr);
    }
    Ok(())
}
```

---

## Module 3: `src/scaffold/generator.rs`

The generator renders Tera templates and copies files into the new project.
All template file contents are embedded at compile time using `include_str!()`.

```rust
use std::{fs, path::Path};
use tera::{Tera, Context};
use anyhow::{Result, Context as _};

pub struct Generator {
    project_dir: String,
    template:    String,
    ctx:         Context,
}

impl Generator {
    pub fn new(project_dir: &str, template: &str) -> Result<Self> {
        let mut ctx = Context::new();
        ctx.insert("project_name", project_dir);
        ctx.insert("template", template);
        ctx.insert("fhevm_version", "0.2.0");
        ctx.insert("relayer_sdk_version", "0.2.0");
        ctx.insert("year", &chrono::Utc::now().format("%Y").to_string());

        Ok(Self {
            project_dir: project_dir.to_string(),
            template: template.to_string(),
            ctx,
        })
    }

    /// Render and write all files for the chosen template.
    /// Also copies shared lib/fhevm/ and agent/ files into every project.
    pub fn render_all(&self) -> Result<()> {
        // 1. Write shared TypeScript SDK files (same for all templates)
        self.write_shared_sdk_files()?;

        // 2. Write template-specific Solidity contracts + tests + scripts
        match self.template.as_str() {
            "blank"   => self.render_template_files(&BLANK_FILES)?,
            "erc7984" => self.render_template_files(&ERC7984_FILES)?,
            "lending" => self.render_template_files(&LENDING_FILES)?,
            "auction" => self.render_template_files(&AUCTION_FILES)?,
            "voting"  => self.render_template_files(&VOTING_FILES)?,
            other     => anyhow::bail!("Unknown template: {}", other),
        }

        Ok(())
    }

    /// Write foundry.toml, fhevm-forge.toml, .env.example, AGENT.md, README.md
    pub fn write_config_files(&self) -> Result<()> {
        self.write_file("foundry.toml",       &self.render_str(FOUNDRY_TOML)?)?;
        self.write_file("fhevm-forge.toml",   FHEVM_FORGE_TOML)?;
        self.write_file(".env.example",       ENV_EXAMPLE)?;
        self.write_file("AGENT.md",           AGENT_MD)?;
        self.write_file("README.md",          &self.render_str(README_MD)?)?;

        // Remove default forge Counter.sol — we provide our own contracts
        let _ = fs::remove_file(
            Path::new(&self.project_dir).join("src/Counter.sol")
        );
        let _ = fs::remove_file(
            Path::new(&self.project_dir).join("test/Counter.t.sol")
        );

        Ok(())
    }

    fn write_shared_sdk_files(&self) -> Result<()> {
        let sdk_files: &[(&str, &str)] = &[
            ("lib/fhevm/instance.ts",         FHEVM_INSTANCE_TS),
            ("lib/fhevm/encrypt.ts",          FHEVM_ENCRYPT_TS),
            ("lib/fhevm/decrypt.ts",          FHEVM_DECRYPT_TS),
            ("lib/fhevm/gateway.ts",          FHEVM_GATEWAY_TS),
            ("lib/fhevm/errors.ts",           FHEVM_ERRORS_TS),
            ("lib/fhevm/config.ts",           FHEVM_CONFIG_TS),
            ("lib/fhevm/index.ts",            FHEVM_INDEX_TS),
            ("lib/hooks/useEncrypt.ts",       HOOK_ENCRYPT_TS),
            ("lib/hooks/useReencrypt.ts",     HOOK_REENCRYPT_TS),
            ("lib/hooks/useHealthCheck.ts",   HOOK_HEALTH_CHECK_TS),
            ("agent/lib/fhevm-agent.ts",      FHEVM_AGENT_TS),
            ("package.json",                  &self.render_str(PACKAGE_JSON)?),
            ("tsconfig.json",                 TSCONFIG_JSON),
        ];

        for (path, content) in sdk_files {
            self.write_file(path, content)?;
        }
        Ok(())
    }

    fn render_template_files(&self, files: &[(&str, &str)]) -> Result<()> {
        for (path, content) in files {
            let rendered = self.render_str(content)?;
            self.write_file(path, &rendered)?;
        }
        Ok(())
    }

    fn render_str(&self, template_str: &str) -> Result<String> {
        Tera::one_off(template_str, &self.ctx, false)
            .map_err(|e| anyhow::anyhow!("Template rendering error: {}", e))
    }

    fn write_file(&self, relative_path: &str, content: &str) -> Result<()> {
        let full_path = Path::new(&self.project_dir).join(relative_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Could not create dir: {}", parent.display()))?;
        }
        fs::write(&full_path, content)
            .with_context(|| format!("Could not write: {}", full_path.display()))?;
        Ok(())
    }
}

// ─── Embedded Template Contents ───────────────────────────────────────────────
// All template files are embedded at compile time.
// Tera templates use {{ variable }} syntax.
// Files without variables are copied as-is.

// Shared config files
const FOUNDRY_TOML:     &str = include_str!("../../templates/shared/foundry.toml.tera");
const FHEVM_FORGE_TOML: &str = include_str!("../../templates/shared/fhevm-forge.toml");
const ENV_EXAMPLE:      &str = include_str!("../../templates/shared/.env.example");
const AGENT_MD:         &str = include_str!("../../templates/shared/AGENT.md");
const README_MD:        &str = include_str!("../../templates/shared/README.md.tera");
const PACKAGE_JSON:     &str = include_str!("../../templates/shared/package.json.tera");
const TSCONFIG_JSON:    &str = include_str!("../../templates/shared/tsconfig.json");

// Shared TypeScript SDK files
const FHEVM_INSTANCE_TS:     &str = include_str!("../../templates/shared/lib/fhevm/instance.ts");
const FHEVM_ENCRYPT_TS:      &str = include_str!("../../templates/shared/lib/fhevm/encrypt.ts");
const FHEVM_DECRYPT_TS:      &str = include_str!("../../templates/shared/lib/fhevm/decrypt.ts");
const FHEVM_GATEWAY_TS:      &str = include_str!("../../templates/shared/lib/fhevm/gateway.ts");
const FHEVM_ERRORS_TS:       &str = include_str!("../../templates/shared/lib/fhevm/errors.ts");
const FHEVM_CONFIG_TS:       &str = include_str!("../../templates/shared/lib/fhevm/config.ts");
const FHEVM_INDEX_TS:        &str = include_str!("../../templates/shared/lib/fhevm/index.ts");
const HOOK_ENCRYPT_TS:       &str = include_str!("../../templates/shared/hooks/useEncrypt.ts");
const HOOK_REENCRYPT_TS:     &str = include_str!("../../templates/shared/hooks/useReencrypt.ts");
const HOOK_HEALTH_CHECK_TS:  &str = include_str!("../../templates/shared/hooks/useHealthCheck.ts");
const FHEVM_AGENT_TS:        &str = include_str!("../../templates/shared/agent/fhevm-agent.ts");

// Template: blank
const BLANK_FILES: &[(&str, &str)] = &[
    ("src/Counter.sol",      include_str!("../../templates/blank/src/Counter.sol")),
    ("test/Counter.t.sol",   include_str!("../../templates/blank/test/Counter.t.sol")),
];

// Template: erc7984
const ERC7984_FILES: &[(&str, &str)] = &[
    ("src/ConfidentialToken.sol",      include_str!("../../templates/erc7984/src/ConfidentialToken.sol")),
    ("test/ConfidentialToken.t.sol",   include_str!("../../templates/erc7984/test/ConfidentialToken.t.sol")),
    ("script/Deploy.s.sol",            include_str!("../../templates/erc7984/script/Deploy.s.sol")),
];

// Template: lending
const LENDING_FILES: &[(&str, &str)] = &[
    ("src/ConfidentialVault.sol",              include_str!("../../templates/lending/src/ConfidentialVault.sol")),
    ("src/tokens/ConfidentialCollateral.sol",  include_str!("../../templates/lending/src/tokens/ConfidentialCollateral.sol")),
    ("src/tokens/ConfidentialDebt.sol",        include_str!("../../templates/lending/src/tokens/ConfidentialDebt.sol")),
    ("src/PriceOracle.sol",                    include_str!("../../templates/lending/src/PriceOracle.sol")),
    ("test/ConfidentialVault.t.sol",           include_str!("../../templates/lending/test/ConfidentialVault.t.sol")),
    ("script/Deploy.s.sol",                    include_str!("../../templates/lending/script/Deploy.s.sol")),
];

// Template: auction
const AUCTION_FILES: &[(&str, &str)] = &[
    ("src/BlindAuction.sol",      include_str!("../../templates/auction/src/BlindAuction.sol")),
    ("test/BlindAuction.t.sol",   include_str!("../../templates/auction/test/BlindAuction.t.sol")),
    ("script/Deploy.s.sol",       include_str!("../../templates/auction/script/Deploy.s.sol")),
];

// Template: voting
const VOTING_FILES: &[(&str, &str)] = &[
    ("src/ConfidentialVoting.sol",      include_str!("../../templates/voting/src/ConfidentialVoting.sol")),
    ("test/ConfidentialVoting.t.sol",   include_str!("../../templates/voting/test/ConfidentialVoting.t.sol")),
    ("script/Deploy.s.sol",             include_str!("../../templates/voting/script/Deploy.s.sol")),
];
```

---

## Module 4: `src/linter/rules/mod.rs`

Define the shared trait and error type all rules implement.

```rust
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct LintError {
    pub rule_id:  String,
    pub severity: Severity,
    pub file:     String,
    pub line:     usize,
    pub message:  String,
    pub snippet:  Option<String>,  // the offending source line, for context
}

pub trait LintRule: Send + Sync {
    fn id(&self) -> &'static str;
    fn check(&self, file_path: &Path, source: &str) -> Vec<LintError>;
}

// Helper: create a LintError concisely
pub fn make_error(
    rule_id: &str,
    severity: Severity,
    file: &Path,
    line: usize,
    message: &str,
    snippet: Option<&str>,
) -> LintError {
    LintError {
        rule_id:  rule_id.to_string(),
        severity,
        file:     file.display().to_string(),
        line,
        message:  message.to_string(),
        snippet:  snippet.map(|s| s.trim().to_string()),
    }
}

// Re-export all rule types
pub use super::rules::missing_allow_this::MissingAllowThis;
pub use super::rules::missing_allow_addr::MissingAllowAddr;
pub use super::rules::view_fhe_function::ViewFheFunction;
pub use super::rules::euint_overflow::EuintOverflow;
pub use super::rules::input_proof_reuse::InputProofReuse;
pub use super::rules::get_relayer_call::GetRelayerCall;
pub use super::rules::missing_only_gateway::MissingOnlyGateway;
pub use super::rules::handle_index_oob::HandleIndexOob;
pub use super::rules::resolver_arg_count::ResolverArgCount;
```

---

## Module 5: Lint Rule Implementations

### `src/linter/rules/missing_allow_this.rs`

```rust
// FHEVM-001: euint* assigned but TFHE.allowThis() not called in same block.
//
// Pattern: Find lines where a euint variable is assigned (either from
// TFHE.asEuint* or TFHE.add/sub/mul/etc.) but TFHE.allowThis() is not
// called within the next 5 lines.
//
// This is intentionally conservative — prefer false negatives over false
// positives. Only flag cases where assignment is obvious.

use regex::Regex;
use std::{path::Path, sync::OnceLock};
use super::{LintError, LintRule, Severity, make_error};

static ASSIGNMENT_RE: OnceLock<Regex> = OnceLock::new();
static ALLOW_THIS_RE: OnceLock<Regex> = OnceLock::new();

pub struct MissingAllowThis;

impl LintRule for MissingAllowThis {
    fn id(&self) -> &'static str { "FHEVM-001" }

    fn check(&self, file_path: &Path, source: &str) -> Vec<LintError> {
        let assignment_re = ASSIGNMENT_RE.get_or_init(|| {
            // Matches: euint64 foo = TFHE.asEuint64(...) or TFHE.add(...) etc.
            Regex::new(r"euint\d+\s+(\w+)\s*=\s*TFHE\.").unwrap()
        });
        let allow_this_re = ALLOW_THIS_RE.get_or_init(|| {
            Regex::new(r"TFHE\.allowThis\(").unwrap()
        });

        let lines: Vec<&str> = source.lines().collect();
        let mut errors = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if let Some(cap) = assignment_re.captures(line) {
                let var_name = &cap[1];

                // Look at the next 6 lines for allowThis
                let window_end = (i + 6).min(lines.len());
                let window = &lines[i..window_end];
                let has_allow_this = window.iter().any(|l| allow_this_re.is_match(l));

                if !has_allow_this {
                    errors.push(make_error(
                        "FHEVM-001",
                        Severity::Error,
                        file_path,
                        i + 1,
                        &format!(
                            "euint variable '{}' assigned but TFHE.allowThis() not called in same scope. \
                             The contract cannot use this handle in future transactions.",
                            var_name
                        ),
                        Some(line),
                    ));
                }
            }
        }

        errors
    }
}
```

### `src/linter/rules/view_fhe_function.rs`

```rust
// FHEVM-003: Function marked view/pure but calls a TFHE operation.
// FHE ops write to internal state registers — they cannot be view functions.

use regex::Regex;
use std::{path::Path, sync::OnceLock};
use super::{LintError, LintRule, Severity, make_error};

static FN_DECL_RE: OnceLock<Regex> = OnceLock::new();
static TFHE_OP_RE: OnceLock<Regex> = OnceLock::new();

pub struct ViewFheFunction;

impl LintRule for ViewFheFunction {
    fn id(&self) -> &'static str { "FHEVM-003" }

    fn check(&self, file_path: &Path, source: &str) -> Vec<LintError> {
        let fn_decl_re = FN_DECL_RE.get_or_init(|| {
            // Matches function declarations containing view or pure modifier
            Regex::new(r"function\s+(\w+)\s*\([^)]*\)[^{]*\b(view|pure)\b[^{]*\{").unwrap()
        });
        let tfhe_op_re = TFHE_OP_RE.get_or_init(|| {
            Regex::new(r"TFHE\.(add|sub|mul|div|lt|le|gt|ge|eq|select|and|or|not|asEuint)").unwrap()
        });

        let source_str = source;
        let lines: Vec<&str> = source_str.lines().collect();
        let mut errors = Vec::new();

        // Find all view/pure function declarations
        for (i, line) in lines.iter().enumerate() {
            if let Some(cap) = fn_decl_re.captures(line) {
                let fn_name = &cap[1];

                // Scan the function body (next 50 lines — simple heuristic)
                let body_end = (i + 50).min(lines.len());
                let body = lines[i..body_end].join("\n");

                if tfhe_op_re.is_match(&body) {
                    errors.push(make_error(
                        "FHEVM-003",
                        Severity::Error,
                        file_path,
                        i + 1,
                        &format!(
                            "Function '{}' is marked 'view' or 'pure' but calls a TFHE operation. \
                             FHE operations modify internal state and cannot be view functions. \
                             Remove the 'view' or 'pure' modifier.",
                            fn_name
                        ),
                        Some(line),
                    ));
                }
            }
        }

        errors
    }
}
```

### `src/linter/rules/get_relayer_call.rs`

```rust
// FHEVM-007: .getRelayer() called — this method does not exist on FhevmInstance.
// Developers assume fhe.getRelayer().publicDecrypt() is the API.
// The correct API is fhe.publicDecrypt() directly.
// This rule scans TypeScript files (*.ts, *.tsx) not Solidity.

use regex::Regex;
use std::{path::Path, sync::OnceLock};
use super::{LintError, LintRule, Severity, make_error};

static GET_RELAYER_RE: OnceLock<Regex> = OnceLock::new();

pub struct GetRelayerCall;

impl LintRule for GetRelayerCall {
    fn id(&self) -> &'static str { "FHEVM-007" }

    fn check(&self, file_path: &Path, source: &str) -> Vec<LintError> {
        // Only apply to TypeScript files
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "ts" && ext != "tsx" { return vec![]; }

        let re = GET_RELAYER_RE.get_or_init(|| {
            Regex::new(r"\.getRelayer\(\)").unwrap()
        });

        source
            .lines()
            .enumerate()
            .filter(|(_, line)| re.is_match(line))
            .map(|(i, line)| {
                make_error(
                    "FHEVM-007",
                    Severity::Error,
                    file_path,
                    i + 1,
                    "Called .getRelayer() on FhevmInstance — this method does not exist. \
                     Use fhe.publicDecrypt() directly instead of fhe.getRelayer().publicDecrypt().",
                    Some(line),
                )
            })
            .collect()
    }
}
```

### `src/linter/rules/missing_only_gateway.rs`

```rust
// FHEVM-006: Gateway callback function missing onlyGateway modifier.
// Any address could call an unprotected callback and inject fake decryption results.

use regex::Regex;
use std::{path::Path, sync::OnceLock};
use super::{LintError, LintRule, Severity, make_error};

static CALLBACK_RE:    OnceLock<Regex> = OnceLock::new();
static ONLY_GATEWAY_RE: OnceLock<Regex> = OnceLock::new();

pub struct MissingOnlyGateway;

impl LintRule for MissingOnlyGateway {
    fn id(&self) -> &'static str { "FHEVM-006" }

    fn check(&self, file_path: &Path, source: &str) -> Vec<LintError> {
        // Matches function names that look like Gateway callbacks
        // Convention: functions taking a uint256 requestId + bool/uint param
        let callback_re = CALLBACK_RE.get_or_init(|| {
            Regex::new(
                r"function\s+(_\w+Decrypted|_on\w+|callbackDecrypt\w*)\s*\([^)]*uint256[^)]*\)"
            ).unwrap()
        });
        let only_gateway_re = ONLY_GATEWAY_RE.get_or_init(|| {
            Regex::new(r"\bonlyGateway\b").unwrap()
        });

        let lines: Vec<&str> = source.lines().collect();
        let mut errors = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if callback_re.is_match(line) && !only_gateway_re.is_match(line) {
                errors.push(make_error(
                    "FHEVM-006",
                    Severity::Error,
                    file_path,
                    i + 1,
                    "Gateway callback function is missing the 'onlyGateway' modifier. \
                     Without this, any address can call the callback and inject arbitrary \
                     decryption results. Add 'onlyGateway' or inherit GatewayCallbackReceiver.",
                    Some(line),
                ));
            }
        }

        errors
    }
}
```

Implement the remaining rules following the same pattern:
- `missing_allow_addr.rs` — FHEVM-002: detect `TFHE.allow()` missing before external call
- `euint_overflow.rs` — FHEVM-004: detect `euint64` in `TFHE.mul()` calls
- `input_proof_reuse.rs` — FHEVM-005: detect same `inputProof` variable used twice
- `handle_index_oob.rs` — FHEVM-008: detect `handles[N]` where N looks out of range
- `resolver_arg_count.rs` — FHEVM-009: detect resolve calls with only 1 arg

---

## Module 6: `src/linter/mod.rs`

```rust
use std::{fs, path::Path};
use anyhow::Result;
use walkdir::WalkDir;

pub mod rules;
pub mod reporter;

use rules::{LintError, LintRule};

pub struct Linter {
    rules: Vec<Box<dyn LintRule>>,
}

impl Linter {
    pub fn new() -> Self {
        Self {
            rules: vec![
                Box::new(rules::MissingAllowThis),
                Box::new(rules::MissingAllowAddr),
                Box::new(rules::ViewFheFunction),
                Box::new(rules::EuintOverflow),
                Box::new(rules::InputProofReuse),
                Box::new(rules::GetRelayerCall),
                Box::new(rules::MissingOnlyGateway),
                Box::new(rules::HandleIndexOob),
                Box::new(rules::ResolverArgCount),
            ],
        }
    }

    pub fn analyze_path(&self, base_path: &str) -> Result<Vec<LintError>> {
        let mut all_errors: Vec<LintError> = Vec::new();

        for entry in WalkDir::new(base_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let ext = e.path().extension()
                    .and_then(|x| x.to_str())
                    .unwrap_or("");
                // Lint .sol files (Solidity rules) and .ts files (SDK misuse rules)
                ext == "sol" || ext == "ts" || ext == "tsx"
            })
        {
            let source = fs::read_to_string(entry.path())?;
            for rule in &self.rules {
                let errors = rule.check(entry.path(), &source);
                all_errors.extend(errors);
            }
        }

        // Sort by file then line number
        all_errors.sort_by(|a, b| {
            a.file.cmp(&b.file).then(a.line.cmp(&b.line))
        });

        Ok(all_errors)
    }
}
```

---

## Module 7: `src/commands/lint.rs`

```rust
use anyhow::Result;
use colored::Colorize;
use crate::linter::{Linter, rules::Severity};

pub async fn run(path: &str, fix: bool) -> Result<()> {
    println!("\n{} {}\n", "Analyzing".cyan().bold(), path.yellow());

    let linter = Linter::new();
    let errors = linter.analyze_path(path)?;

    if errors.is_empty() {
        println!("{}", "✅ No FHEVM issues found.".green().bold());
        return Ok(());
    }

    // Group by file
    let mut by_file: std::collections::BTreeMap<String, Vec<&crate::linter::rules::LintError>> =
        std::collections::BTreeMap::new();

    for e in &errors {
        by_file.entry(e.file.clone()).or_default().push(e);
    }

    for (file, file_errors) in &by_file {
        println!("  {}", file.cyan());
        println!("  {}", "─".repeat(60).dimmed());

        for e in file_errors {
            let severity_str = match e.severity {
                Severity::Error   => "ERROR".red().bold(),
                Severity::Warning => "WARN ".yellow().bold(),
            };
            println!(
                "  [{}] {}  Line {:>4}  — {}",
                e.rule_id.dimmed(),
                severity_str,
                e.line,
                e.message,
            );
            if let Some(snippet) = &e.snippet {
                println!("              {}", snippet.dimmed());
            }
        }
        println!();
    }

    let error_count   = errors.iter().filter(|e| e.severity == Severity::Error).count();
    let warning_count = errors.iter().filter(|e| e.severity == Severity::Warning).count();

    println!(
        "{}: {} error{}, {} warning{}",
        "Summary".bold(),
        error_count.to_string().red(),
        if error_count == 1 { "" } else { "s" },
        warning_count.to_string().yellow(),
        if warning_count == 1 { "" } else { "s" },
    );

    if fix {
        println!("\n{}", "Auto-fix mode is not yet implemented for all rules.".yellow());
        println!("Safe auto-fixes (FHEVM-001, FHEVM-003) will be added in v0.2.0.");
    } else {
        println!(
            "\nRun {} to attempt auto-fix of safe issues.",
            "fhevm-forge lint --fix".cyan()
        );
    }

    // Exit with non-zero code if there are errors (for CI integration)
    if error_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}
```

---

## Module 8: `src/deployer/chains.rs`

```rust
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
        // Addresses populated when Zama deploys to mainnet
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
```

---

## Module 9: `src/commands/deploy.rs`

```rust
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

        // Check RPC URL env var is set
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
        // Write deployment manifest
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

    // Inject chain-specific FHEVM addresses as env vars
    // Forge scripts read these to configure the contract during deployment
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

    // Parse contract address and tx hash from forge output
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
    // Forge prints: "Contract Address: 0x..."
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
```

---

## Module 10: `src/commands/gas.rs`

```rust
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;

// FHE operation cost table
// (on_chain_gas, coprocessor_gas)
fn fhe_costs() -> HashMap<&'static str, (u64, u64)> {
    let mut m = HashMap::new();
    m.insert("TFHE.add",                    (8_000,  65_000));
    m.insert("TFHE.sub",                    (8_000,  65_000));
    m.insert("TFHE.mul",                    (15_000, 150_000));
    m.insert("TFHE.div",                    (30_000, 400_000));
    m.insert("TFHE.lt",                     (10_000, 70_000));
    m.insert("TFHE.le",                     (10_000, 70_000));
    m.insert("TFHE.gt",                     (10_000, 70_000));
    m.insert("TFHE.ge",                     (10_000, 70_000));
    m.insert("TFHE.eq",                     (10_000, 70_000));
    m.insert("TFHE.select",                 (12_000, 90_000));
    m.insert("TFHE.and",                    (5_000,  30_000));
    m.insert("TFHE.or",                     (5_000,  30_000));
    m.insert("TFHE.not",                    (5_000,  30_000));
    m.insert("TFHE.asEuint64",              (6_000,  50_000));
    m.insert("TFHE.asEuint128",             (6_500,  55_000));
    m.insert("TFHE.allow",                  (3_000,  0));
    m.insert("TFHE.allowThis",              (3_000,  0));
    m.insert("Gateway.requestDecryption",   (25_000, 200_000));
    m
}

pub async fn run(contract: Option<&str>, output_format: &str) -> Result<()> {
    // Run forge test with gas reporting
    println!("{} forge test --gas-report...", "Running".cyan().bold());

    let mut cmd = tokio::process::Command::new("forge");
    cmd.args(["test", "--gas-report"]);
    if let Some(c) = contract {
        cmd.args(["--match-contract", c]);
    }

    let output = cmd.output().await?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Count TFHE op occurrences in test output + source files
    let op_counts = count_fhe_ops_in_source("./src")?;
    let costs = fhe_costs();

    match output_format {
        "json"     => print_json_report(&op_counts, &costs)?,
        "markdown" => print_markdown_report(&op_counts, &costs),
        _          => print_terminal_report(&op_counts, &costs),
    }

    // Also show the standard forge gas table
    if !stdout.is_empty() {
        println!("\n{}", "Standard Forge Gas Report:".dimmed());
        println!("{}", stdout.dimmed());
    }

    Ok(())
}

fn count_fhe_ops_in_source(path: &str) -> Result<HashMap<String, u64>> {
    use std::fs;
    use walkdir::WalkDir;
    use regex::Regex;

    let mut counts: HashMap<String, u64> = HashMap::new();
    let op_re = Regex::new(r"TFHE\.(add|sub|mul|div|lt|le|gt|ge|eq|select|and|or|not|asEuint64|asEuint128|allow|allowThis)|Gateway\.requestDecryption")?;

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "sol").unwrap_or(false))
    {
        let source = fs::read_to_string(entry.path())?;
        for cap in op_re.find_iter(&source) {
            *counts.entry(cap.as_str().to_string()).or_insert(0) += 1;
        }
    }

    Ok(counts)
}

fn print_terminal_report(counts: &HashMap<String, u64>, costs: &HashMap<&str, (u64, u64)>) {
    println!("\n{}", "⛽ FHE Gas Report".cyan().bold());
    println!("{}", "─".repeat(85));
    println!(
        "{:<30} {:<10} {:<18} {:<18} {}",
        "Operation".bold(),
        "Count".bold(),
        "On-Chain Gas".bold(),
        "Coprocessor Gas".bold(),
        "% of FHE Total".bold()
    );
    println!("{}", "─".repeat(85));

    let mut rows: Vec<(String, u64, u64, u64)> = counts
        .iter()
        .filter_map(|(op, &count)| {
            costs.get(op.as_str()).map(|&(on_chain, coprocessor)| {
                (op.clone(), count, on_chain * count, coprocessor * count)
            })
        })
        .collect();

    let total_fhe: u64 = rows.iter().map(|(_, _, on, cop)| on + cop).sum();

    rows.sort_by(|a, b| (b.2 + b.3).cmp(&(a.2 + a.3)));

    for (op, count, on_chain, coprocessor) in &rows {
        let pct = if total_fhe > 0 {
            ((on_chain + coprocessor) as f64 / total_fhe as f64) * 100.0
        } else { 0.0 };

        println!(
            "{:<30} {:<10} {:<18} {:<18} {:.1}%",
            op, count,
            format_gas(*on_chain),
            format_gas(*coprocessor),
            pct
        );
    }

    println!("{}", "─".repeat(85));

    let total_on_chain: u64   = rows.iter().map(|(_, _, on, _)| on).sum();
    let total_coprocessor: u64 = rows.iter().map(|(_, _, _, cop)| cop).sum();
    let evm_baseline: u64     = rows.iter().map(|(_, count, _, _)| count * 100).sum(); // ~100 gas plain op

    println!(
        "{:<30} {:<10} {:<18} {:<18}",
        "TOTAL".bold(),
        "",
        format_gas(total_on_chain).bold().to_string(),
        format_gas(total_coprocessor).bold().to_string(),
    );

    if evm_baseline > 0 {
        let multiplier = (total_on_chain + total_coprocessor) as f64 / evm_baseline as f64;
        println!(
            "\n{} FHE total is ~{:.1}x the cost of equivalent plaintext EVM logic",
            "⚠️ ".yellow(),
            multiplier
        );
    }

    if let Some((most_expensive, _, _, _)) = rows.first() {
        println!(
            "{} Most expensive operation: {} — consider batching or reducing call count",
            "💡".cyan(),
            most_expensive.yellow()
        );
    }
}

fn print_json_report(counts: &HashMap<String, u64>, costs: &HashMap<&str, (u64, u64)>) -> Result<()> {
    let report: Vec<serde_json::Value> = counts.iter()
        .filter_map(|(op, &count)| {
            costs.get(op.as_str()).map(|&(on_chain, coprocessor)| {
                serde_json::json!({
                    "operation": op,
                    "count": count,
                    "on_chain_gas": on_chain * count,
                    "coprocessor_gas": coprocessor * count,
                    "total_gas": (on_chain + coprocessor) * count,
                })
            })
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn print_markdown_report(counts: &HashMap<String, u64>, costs: &HashMap<&str, (u64, u64)>) {
    println!("## FHE Gas Report\n");
    println!("| Operation | Count | On-Chain Gas | Coprocessor Gas |");
    println!("|-----------|-------|-------------|-----------------|");
    for (op, &count) in counts {
        if let Some(&(on_chain, coprocessor)) = costs.get(op.as_str()) {
            println!("| {} | {} | {} | {} |", op, count, on_chain * count, coprocessor * count);
        }
    }
}

fn format_gas(gas: u64) -> String {
    if gas == 0 { return "0".to_string(); }
    // Add thousands separators
    let s = gas.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { result.push(','); }
        result.push(c);
    }
    result.chars().rev().collect()
}
```

---

## Module 11: `src/commands/doctor.rs`

```rust
use anyhow::Result;
use colored::Colorize;
use std::env;

pub async fn run() -> Result<()> {
    println!("\n{}\n", "Checking FHEVM development environment...".cyan().bold());

    let mut all_ok = true;

    // Check forge
    let forge_ok = check_tool("forge", &["--version"]).await;
    print_check("forge", forge_ok, "Foundry not installed. Run: curl -L https://foundry.paradigm.xyz | bash");
    all_ok &= forge_ok;

    // Check node
    let node_ok = check_tool("node", &["--version"]).await;
    print_check("node >= 20", node_ok, "Node.js not installed. Required for TypeScript SDK. Install from https://nodejs.org");
    all_ok &= node_ok;

    // Check if forge-fhevm is installed (look for lib/forge-fhevm directory)
    let fhevm_installed = std::path::Path::new("lib/forge-fhevm").exists();
    print_check(
        "forge-fhevm",
        fhevm_installed,
        "forge-fhevm not installed. Run: forge install zama-ai/forge-fhevm --no-commit"
    );

    // Check foundry.toml has evm_version = "cancun"
    let foundry_ok = check_foundry_toml();
    print_check(
        "foundry.toml evm_version=cancun",
        foundry_ok,
        "foundry.toml missing or evm_version != cancun. forge-fhevm requires cancun EVM."
    );

    println!();

    // Check required env vars
    let env_vars = [
        ("SEPOLIA_RPC_URL",      true,  "Required for Sepolia deployment"),
        ("DEPLOYER_PRIVATE_KEY", true,  "Required for all deployments"),
        ("MAINNET_RPC_URL",      false, "Required for mainnet deployment"),
        ("ETHERSCAN_API_KEY",    false, "Required for contract verification"),
        ("BASESCAN_API_KEY",     false, "Required for Base verification"),
    ];

    for (var, required, hint) in &env_vars {
        let is_set = env::var(var).map(|v| !v.is_empty()).unwrap_or(false);
        let label = format!("{:<30}", var);

        if is_set {
            println!("  {} {}", "✅".green(), label.green());
        } else if *required {
            println!("  {} {} — {}", "❌".red(), label.red(), hint.dimmed());
            all_ok = false;
        } else {
            println!("  {} {} — {} {}", "⚠️ ".yellow(), label.yellow(), "not set".dimmed(), format!("({})", hint).dimmed());
        }
    }

    println!();
    if all_ok {
        println!("{}", "✅ Environment is ready for FHEVM development.".green().bold());
    } else {
        println!("{}", "❌ Fix the issues above before deploying.".red().bold());
    }

    Ok(())
}

async fn check_tool(tool: &str, args: &[&str]) -> bool {
    tokio::process::Command::new(tool)
        .args(args)
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn print_check(label: &str, ok: bool, hint: &str) {
    let label = format!("{:<40}", label);
    if ok {
        println!("  {} {}", "✅".green(), label.green());
    } else {
        println!("  {} {} — {}", "❌".red(), label.red(), hint.dimmed());
    }
}

fn check_foundry_toml() -> bool {
    std::fs::read_to_string("foundry.toml")
        .map(|s| s.contains("cancun"))
        .unwrap_or(false)
}
```

---

## Integration Tests

### `tests/init_test.rs`

```rust
use assert_cmd::Command;
use tempfile::TempDir;

#[test]
fn test_init_lending_template() {
    let tmp = TempDir::new().unwrap();
    let project_name = "test-lending";

    Command::cargo_bin("fhevm-forge")
        .unwrap()
        .args(["init", project_name, "--template", "lending"])
        .current_dir(tmp.path())
        .assert()
        .success();

    // Verify key files were created
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
```

### `tests/lint_test.rs`

```rust
use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_lint_catches_missing_allow_this() {
    let tmp = TempDir::new().unwrap();
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
fn test_lint_catches_get_relayer_call() {
    let tmp = TempDir::new().unwrap();
    let bad_ts = r#"
import { getFhevmInstance } from './fhevm';
async function decrypt(handles: bigint[]) {
    const fhe = await getFhevmInstance();
    // This is wrong — getRelayer() doesn't exist
    const result = await fhe.getRelayer().publicDecrypt(handles);
}
"#;

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
```
