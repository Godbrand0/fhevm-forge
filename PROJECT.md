# FHEVM Forge: The Ultimate FHE Development Framework

## Overview
**FHEVM Forge** is a high-performance development tool designed to make Fully Homomorphic Encryption (FHE) on the EVM as accessible as standard Solidity development. Built for the Zama FHEVM ecosystem, it combines the speed of Foundry with specialized scaffolding to help developers build truly private decentralized applications.

## Core Philosophy: Batteries-Included FHE
The primary hurdle in FHE development is the "Read" operation—decrypting values securely and efficiently. FHEVM Forge solves this by deeply integrating the **Zama Relayer SDK** into every scaffolded project.

### The "Relayer-Native" Frontend
When you scaffold a project with the `--frontend` flag, the Next.js application is "Relayer-Native" out of the box. This provides:

*   **Zero-Config Decryption**: We handle the complex WebAssembly (WASM) initialization and environment polyfills (crypto, buffer) in the bundler configuration so you don't have to.
*   **User-Centric Privacy**: Built-in React hooks (`useReencrypt`) allow users to reveal their own private data (like account balances or voting choices) with a simple EIP-712 wallet signature.
*   **Public Decryption Support**: Automated polling and proof resolution for contract-initiated decryptions, abstracting the multi-chain (Host vs. Gateway) complexity.

## Monorepo Architecture
FHEVM Forge projects are structured as modern **npm workspace monorepos**. This architecture ensures that complex cryptographic logic can be shared seamlessly across all parts of your application without code duplication.

*   **`@fhevm/sdk`**: The shared source of truth. Contains all FHEVM logic, WASM initialization, and unified React hooks used by both the UI and background agents.
*   **`@fhevm/agent`**: A clean, headless workspace for autonomous scripts, keepers, and monitor agents.
*   **`@fhevm/frontend`**: A Next.js 15 application optimized for private data interaction.

## Streamlined Development Experience (DX)
To ensure the fastest "Time to First Transaction," the framework provides a clean, robust starting point:

1.  **Workspaces Established**: Run `npm install` at the root once to link all packages. Use `npm run build` or `npm run typecheck` to manage the entire stack simultaneously.
2.  **The Counter Template**: A canonical FHE example that demonstrates the entire lifecycle of encrypted data.
3.  **FHE-Aware Tooling**:
    *   **FHE Linting**: Custom rules (e.g., `FHEVM-001`) to catch common FHE security pitfalls.
    *   **Gas Profiling**: Per-operation breakdown of FHE coprocessor costs.
4.  **Clean Imports**: Use standard package imports (e.g., `import { useFhevm } from "@fhevm/sdk"`) for a professional and maintainable codebase.

## The Mission
FHE is the next frontier of blockchain privacy. FHEVM Forge provides the bridge for developers to cross that frontier without needing to be cryptography experts. We manage the underlying complexity of threshold decryption and monorepo management so you can focus on building the next generation of private dApps.
