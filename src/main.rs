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

        /// Comma-separated rule IDs to suppress (e.g. FHEVM-001,FHEVM-003)
        #[arg(long, value_delimiter = ',')]
        ignore: Vec<String>,

        /// Print all available rule IDs and exit
        #[arg(long)]
        list_rules: bool,
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
        Commands::Lint { path, fix, ignore, list_rules } => {
            commands::lint::run(&path, fix, ignore, list_rules).await
        }
        Commands::Doctor => {
            commands::doctor::run().await
        }
    }
}
