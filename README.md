# fhevm-forge

Foundry scaffold, deployer, gas estimator and linter for [Zama FHEVM](https://docs.zama.ai/fhevm) — the Fully Homomorphic Encryption Virtual Machine.

```bash
cargo install fhevm-forge
fhevm-forge init my-protocol --template lending
```

---

## What it does

`fhevm-forge` is a CLI tool that helps you build confidential smart contracts on Zama FHEVM. It handles the boilerplate so you can focus on writing encrypted business logic.

| Command | What it does |
|---|---|
| `fhevm-forge init` | Scaffold a new Foundry project with FHEVM contracts, TypeScript SDK, and agent helpers |
| `fhevm-forge lint` | Static analysis for common FHEVM bugs (missing `allowThis`, wrong resolver args, etc.) |
| `fhevm-forge gas` | FHE-aware gas cost report — shows on-chain + coprocessor gas per operation |
| `fhevm-forge deploy` | Deploy contracts to Sepolia, mainnet, Base, or Arbitrum with one command |
| `fhevm-forge doctor` | Check your development environment |

---

## Installation

```bash
cargo install fhevm-forge
```

Foundry is required for `init` and `deploy`:

```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

---

## Quickstart

```bash
# Scaffold a confidential lending vault
fhevm-forge init my-vault --template lending
cd my-vault

# Run tests (uses forge-fhevm local mock — no Gateway needed)
forge test

# Check for FHEVM-specific bugs
fhevm-forge lint ./src

# See FHE gas costs
fhevm-forge gas

# Deploy to Sepolia
cp .env.example .env   # fill in RPC URL and private key
fhevm-forge deploy --chains sepolia --contract ConfidentialVault
```

---

## Templates

| Template | Description |
|---|---|
| `blank` | Bare Foundry project with FHEVM SDK |
| `erc7984` | Confidential ERC-7984 token |
| `lending` | Confidential lending vault (cETH collateral, cUSDC debt) |
| `auction` | Blind Dutch auction with encrypted bids |
| `voting` | Confidential voting system |

```bash
fhevm-forge init my-project --template <name>
# or omit --template for an interactive selector
fhevm-forge init my-project
```

---

## What gets generated

Every scaffolded project includes:

```
my-project/
├── src/                        Solidity contracts (TFHE encrypted types)
├── test/                       Forge tests (local FHE mock, no Gateway)
├── script/                     Deployment scripts
├── lib/
│   ├── forge-std/              Foundry standard library
│   └── forge-fhevm/            Zama FHE mock for local testing
├── AGENT.md                    FHEVM development guide for AI coding agents
├── foundry.toml
├── fhevm-forge.toml
├── package.json                fhevm-forge-sdk + ethers
└── tsconfig.json
```

---

## Lint rules

The linter catches bugs that compile fine but fail silently at runtime:

| Rule | What it catches |
|---|---|
| `FHEVM-001` | `euint` assigned but `TFHE.allowThis()` not called |
| `FHEVM-002` | Handle passed to external contract without `TFHE.allow()` |
| `FHEVM-003` | FHE operation inside a `view` function |
| `FHEVM-005` | `inputProof` reused across multiple contracts |
| `FHEVM-006` | Gateway callback missing `onlyGateway` modifier |
| `FHEVM-007` | `fhe.getRelayer().publicDecrypt()` — `getRelayer()` does not exist |
| `FHEVM-008` | Handle accessed by index beyond getter return count |
| `FHEVM-009` | Resolver called with wrong number of arguments |

```bash
fhevm-forge lint ./src
fhevm-forge lint ./src --ignore FHEVM-001,FHEVM-003
fhevm-forge lint --list-rules
```

---

## Gas report

```bash
fhevm-forge gas
fhevm-forge gas --output json
fhevm-forge gas --output markdown
```

Reports both on-chain EVM gas and Zama coprocessor gas per TFHE operation found in your source files:

```
⛽ FHE Gas Report
─────────────────────────────────────────────────────────────────────────────────────
Operation                      Count      On-Chain Gas       Coprocessor Gas    % of FHE Total
─────────────────────────────────────────────────────────────────────────────────────
TFHE.mul                       4          60,000             600,000            72.1%
TFHE.add                       3          24,000             195,000            21.5%
TFHE.allowThis                 7          21,000             0                  2.5%
```

---

## Deploy

```bash
# Single chain
fhevm-forge deploy --chains sepolia --contract ConfidentialVault

# Multiple chains
fhevm-forge deploy --chains sepolia,base,arbitrum --contract ConfidentialVault

# Dry run (simulate without broadcasting)
fhevm-forge deploy --chains sepolia --contract ConfidentialVault --dry-run
```

Deployment manifests are written to `deployments/<ContractName>.json`.

---

## Environment variables

```bash
SEPOLIA_RPC_URL=          # Required for Sepolia deployment
DEPLOYER_PRIVATE_KEY=     # Wallet deploying contracts
ETHERSCAN_API_KEY=        # For contract verification
BASE_RPC_URL=
ARBITRUM_RPC_URL=
```

---

## Requirements

- Rust 1.70+
- [Foundry](https://getfoundry.sh) (for `init` and `deploy`)
- Node.js (for TypeScript SDK in scaffolded projects)

---

## License

MIT
