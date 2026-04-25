# fhevm-forge — Template File Contents
> This file contains the complete contents of every file in the `templates/` directory.
> These files are embedded in the binary via `include_str!()` in `scaffold/generator.rs`.
> When adding a new template file: write its content here first, then add it to the
> generator's `include_str!()` table and the appropriate file list constant.

---

## Shared Config Templates

### `templates/shared/foundry.toml.tera`

```toml
[profile.default]
src         = "src"
out         = "out"
libs        = ["lib"]
solc        = "0.8.27"
evm_version = "cancun"

[profile.default.fuzz]
runs = 256

[profile.ci]
fuzz = { runs = 1000 }

[rpc_endpoints]
sepolia  = "${SEPOLIA_RPC_URL}"
mainnet  = "${MAINNET_RPC_URL}"
base     = "${BASE_RPC_URL}"
arbitrum = "${ARBITRUM_RPC_URL}"

[etherscan]
sepolia  = { key = "${ETHERSCAN_API_KEY}", url = "https://api-sepolia.etherscan.io/api" }
mainnet  = { key = "${ETHERSCAN_API_KEY}" }
base     = { key = "${BASESCAN_API_KEY}",  url = "https://api.basescan.org/api" }
arbitrum = { key = "${ARBISCAN_API_KEY}",  url = "https://api.arbiscan.io/api" }
```

### `templates/shared/fhevm-forge.toml`

```toml
# fhevm-forge configuration
# See: https://github.com/yourusername/fhevm-forge

[deploy]
chains           = ["sepolia"]
default_contract = "ConfidentialVault"

[lint]
# Files/directories to skip
ignore = ["lib/**", "test/**", "script/**"]

[lint.rules]
# FHEVM-001: euint assigned without TFHE.allowThis()
FHEVM-001 = "error"
# FHEVM-002: euint passed to external contract without TFHE.allow()
FHEVM-002 = "error"
# FHEVM-003: view/pure function contains TFHE operation
FHEVM-003 = "error"
# FHEVM-004: euint64 in TFHE.mul() — overflow risk
FHEVM-004 = "warning"
# FHEVM-005: Same inputProof used in multiple TFHE.asEuint() calls
FHEVM-005 = "error"
# FHEVM-006: Gateway callback missing onlyGateway modifier
FHEVM-006 = "error"
# FHEVM-007: fhe.getRelayer() called — method does not exist
FHEVM-007 = "error"
# FHEVM-008: handles[N] accessed where N exceeds getter return count
FHEVM-008 = "error"
# FHEVM-009: Resolver called with wrong number of arguments
FHEVM-009 = "error"

[gas]
output    = ["terminal", "json"]
json_path = "./gas-report.json"
# Warn if total FHE coprocessor cost exceeds this threshold
warn_if_coprocessor_gas_exceeds = 5_000_000
```

### `templates/shared/.env.example`

```bash
# Network RPC endpoints
SEPOLIA_RPC_URL=
MAINNET_RPC_URL=
BASE_RPC_URL=
ARBITRUM_RPC_URL=

# Wallet private keys (use separate keys for each role)
DEPLOYER_PRIVATE_KEY=
MONITOR_AGENT_PRIVATE_KEY=
BIDDER_AGENT_PRIVATE_KEY=

# Explorer API keys (for contract verification)
ETHERSCAN_API_KEY=
BASESCAN_API_KEY=
ARBISCAN_API_KEY=
```

### `templates/shared/tsconfig.json`

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "outDir": "dist",
    "rootDir": ".",
    "paths": {
      "@/*": ["./*"]
    }
  },
  "include": ["lib/**/*", "agent/**/*"],
  "exclude": ["node_modules", "dist"]
}
```

### `templates/shared/package.json.tera`

```json
{
  "name": "{{ project_name }}",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "typecheck": "tsc --noEmit",
    "agent:monitor": "tsx agent/monitor.ts",
    "agent:bidder":  "tsx agent/bidder.ts"
  },
  "dependencies": {
    "@zama-fhe/relayer-sdk": "^{{ relayer_sdk_version }}",
    "ethers": "^6.0.0"
  },
  "devDependencies": {
    "typescript": "^5.0.0",
    "tsx": "^4.0.0",
    "@types/node": "^20.0.0"
  }
}
```

---

## Shared TypeScript SDK Files

### `templates/shared/lib/fhevm/instance.ts`

```typescript
import { createInstance, FhevmInstance } from "@zama-fhe/relayer-sdk";
import { CHAIN_CONFIGS, type ChainKey } from "./config";

// ─── Singleton Instance Management ───────────────────────────────────────────
//
// The FhevmInstance is expensive to create — it fetches the FHE public key
// from the chain. Create it once and reuse across all operations.
//
// Never call createInstance() directly in application code.
// Always use getFhevmInstance() from this file.

let _instance: FhevmInstance | null = null;
let _currentChain: ChainKey | null  = null;

type Environment = "browser" | "node" | "test";

function detectEnvironment(): Environment {
  if (typeof process !== "undefined" && process.env.NODE_ENV === "test") return "test";
  if (typeof window  !== "undefined") return "browser";
  return "node";
}

/**
 * Returns the singleton FhevmInstance for the given chain.
 * Safe to call multiple times — initializes only once per chain.
 *
 * @param chain    Chain to connect to. Default: "sepolia"
 * @param provider Optional ethers/viem provider (required in browser for wallet ops)
 */
export async function getFhevmInstance(
  chain: ChainKey = "sepolia",
  provider?: unknown
): Promise<FhevmInstance> {
  // Re-initialize if chain changed (e.g. user switched network in wallet)
  if (_instance && _currentChain === chain) return _instance;

  const config = CHAIN_CONFIGS[chain];
  if (!config) throw new Error(`Unknown chain: ${chain}`);

  const env = detectEnvironment();

  if (env === "test") {
    // Test environment: use local Anvil node running forge-fhevm
    _instance = await createInstance({
      ...config,
      chainId:        31337,
      gatewayChainId: 31337,
    });
  } else {
    _instance = await createInstance({
      ...config,
      ...(provider ? { network: provider } : {}),
    });
  }

  _currentChain = chain;
  return _instance;
}

/**
 * Reset the singleton. Call this when the user switches network in their wallet.
 */
export function resetFhevmInstance(): void {
  _instance     = null;
  _currentChain = null;
}
```

### `templates/shared/lib/fhevm/config.ts`

```typescript
// FHEVM chain configurations.
// All addresses are Zama's official deployed contracts — do not modify.

export type ChainKey = "sepolia" | "mainnet" | "base" | "arbitrum";

export interface FhevmChainConfig {
  chainId:                                  number;
  gatewayChainId:                           number;
  aclContractAddress:                       string;
  kmsContractAddress:                       string;
  inputVerifierContractAddress:             string;
  verifyingContractAddressDecryption:       string;
  verifyingContractAddressInputVerification: string;
}

export const CHAIN_CONFIGS: Record<ChainKey, FhevmChainConfig> = {
  sepolia: {
    chainId:                                  11155111,
    gatewayChainId:                           55815,
    aclContractAddress:                       "0x687820221192C5B662b25367F70076A37bc79b6c",
    kmsContractAddress:                       "0x1364cBBf2cDF5032C47d8226a6f6FBD2AFCDacAC",
    inputVerifierContractAddress:             "0xbc91f3daD1A5F19F8390c400196e58073B6a0BC4",
    verifyingContractAddressDecryption:       "0xb6E160B1ff80D67Bfe90A85eE06Ce0A2613607D1",
    verifyingContractAddressInputVerification:"0x7048C39f048125eDa9d678AEbaDfB22F7900a29F",
  },
  mainnet: {
    chainId: 1, gatewayChainId: 55815,
    // Addresses populated at Zama mainnet launch
    aclContractAddress: "", kmsContractAddress: "",
    inputVerifierContractAddress: "",
    verifyingContractAddressDecryption: "",
    verifyingContractAddressInputVerification: "",
  },
  base: {
    chainId: 8453, gatewayChainId: 55815,
    aclContractAddress: "", kmsContractAddress: "",
    inputVerifierContractAddress: "",
    verifyingContractAddressDecryption: "",
    verifyingContractAddressInputVerification: "",
  },
  arbitrum: {
    chainId: 42161, gatewayChainId: 55815,
    aclContractAddress: "", kmsContractAddress: "",
    inputVerifierContractAddress: "",
    verifyingContractAddressDecryption: "",
    verifyingContractAddressInputVerification: "",
  },
};
```

### `templates/shared/lib/fhevm/errors.ts`

```typescript
// FHE-specific error types and friendly message translation.
// All SDK operations are wrapped with these — raw Relayer SDK errors
// are often cryptic. This layer makes them actionable.

export class FhevmError extends Error {
  constructor(
    message: string,
    public readonly operation?: string,
    public readonly cause?: unknown
  ) {
    super(`[FHEVM${operation ? `:${operation}` : ""}] ${message}`);
    this.name = "FhevmError";
  }
}

// Map of known Relayer SDK error substrings → human-readable guidance
const ERROR_MAP: Array<[string, string]> = [
  [
    "getRelayer is not a function",
    "Called fhe.getRelayer() — this method does not exist. " +
    "Use fhe.publicDecrypt() directly on the FhevmInstance.",
  ],
  [
    "invalid handle",
    "Encrypted handle is 0 or uninitialized. " +
    "Check that TFHE.allowThis() was called after assigning the euint value.",
  ],
  [
    "ACL not authorized",
    "Address does not have TFHE.allow() permission for this handle. " +
    "Call TFHE.allow(handle, address) in the contract before reading.",
  ],
  [
    "input proof",
    "inputProof is invalid or expired. Re-encrypt the value — " +
    "proofs are single-use and bound to one contract + user address pair.",
  ],
  [
    "execution reverted",
    "Contract reverted. Common FHEVM causes: " +
    "(1) missing TFHE.allowThis(), " +
    "(2) wrong arg count on resolver (needs borrower + abiEncodedClearValues + decryptionProof), " +
    "(3) missing onlyGateway modifier on callback, " +
    "(4) inactive position.",
  ],
  [
    "network changed",
    "Wallet network changed. Call resetFhevmInstance() then getFhevmInstance() again.",
  ],
];

/**
 * Wrap an async FHE operation with friendly error messages.
 * Use this in all public-facing SDK helper functions.
 */
export async function wrap<T>(
  operation: string,
  fn: () => Promise<T>
): Promise<T> {
  try {
    return await fn();
  } catch (err: unknown) {
    const raw = err instanceof Error ? err.message : String(err);
    const known = ERROR_MAP.find(([pattern]) => raw.includes(pattern));

    if (known) {
      throw new FhevmError(known[1], operation, err);
    }

    // Re-throw with operation context for unknown errors
    throw new FhevmError(raw, operation, err);
  }
}
```

### `templates/shared/lib/fhevm/encrypt.ts`

```typescript
import { getFhevmInstance }  from "./instance";
import { wrap }              from "./errors";
import type { ChainKey }     from "./config";

export interface EncryptedInput {
  handles:    Uint8Array[];
  inputProof: Uint8Array;
}

type EncryptType =
  | "bool" | "uint8" | "uint16" | "uint32"
  | "uint64" | "uint128" | "uint256" | "address";

/**
 * Encrypt a single value for submission to an FHEVM contract.
 *
 * IMPORTANT: The inputProof is bound to one specific contractAddress + userAddress.
 * Never reuse a proof across different contracts. Create a new encrypted input
 * for each contract you interact with. (Lint rule: FHEVM-005)
 *
 * @param type            The encrypted type to use
 * @param value           Plaintext value
 * @param contractAddress Contract that will receive and verify this encrypted input
 * @param userAddress     Wallet address submitting the transaction
 * @param chain           Chain to encrypt for (default: sepolia)
 */
export async function encryptValue(
  type:            EncryptType,
  value:           bigint | boolean | string,
  contractAddress: string,
  userAddress:     string,
  chain:           ChainKey = "sepolia"
): Promise<EncryptedInput> {
  return wrap("encrypt", async () => {
    const fhe   = await getFhevmInstance(chain);
    const input = fhe.createEncryptedInput(contractAddress, userAddress);

    switch (type) {
      case "bool":    input.addBool(value as boolean);   break;
      case "uint8":   input.add8(value as bigint);       break;
      case "uint16":  input.add16(value as bigint);      break;
      case "uint32":  input.add32(value as bigint);      break;
      case "uint64":  input.add64(value as bigint);      break;
      case "uint128": input.add128(value as bigint);     break;
      case "uint256": input.add256(value as bigint);     break;
      case "address": input.addAddress(value as string); break;
      default:        throw new Error(`Unknown encrypted type: ${type}`);
    }

    return input.encrypt();
  });
}

// Typed convenience shorthands
export const encryptBool    = (v: boolean, c: string, u: string, chain?: ChainKey) =>
  encryptValue("bool",    v, c, u, chain);
export const encryptUint8   = (v: bigint,  c: string, u: string, chain?: ChainKey) =>
  encryptValue("uint8",   v, c, u, chain);
export const encryptUint32  = (v: bigint,  c: string, u: string, chain?: ChainKey) =>
  encryptValue("uint32",  v, c, u, chain);
export const encryptUint64  = (v: bigint,  c: string, u: string, chain?: ChainKey) =>
  encryptValue("uint64",  v, c, u, chain);
export const encryptUint128 = (v: bigint,  c: string, u: string, chain?: ChainKey) =>
  encryptValue("uint128", v, c, u, chain);
export const encryptAddress = (v: string,  c: string, u: string, chain?: ChainKey) =>
  encryptValue("address", v, c, u, chain);

/**
 * Encrypt multiple values in one SDK call.
 * More efficient than multiple encryptValue() calls.
 * All values must target the same contract + user.
 * A single inputProof covers all values in the batch.
 *
 * @example
 * const { handles, inputProof } = await encryptBatch([
 *   { type: "uint64", value: collateralAmountGwei },
 *   { type: "uint64", value: borrowAmountUsdc },
 * ], vaultAddress, userAddress);
 * // handles[0] = collateral handle, handles[1] = borrow handle
 */
export async function encryptBatch(
  inputs:          Array<{ type: EncryptType; value: bigint | boolean | string }>,
  contractAddress: string,
  userAddress:     string,
  chain:           ChainKey = "sepolia"
): Promise<EncryptedInput> {
  return wrap("encryptBatch", async () => {
    const fhe   = await getFhevmInstance(chain);
    const input = fhe.createEncryptedInput(contractAddress, userAddress);

    for (const { type, value } of inputs) {
      switch (type) {
        case "bool":    input.addBool(value as boolean);   break;
        case "uint64":  input.add64(value as bigint);      break;
        case "uint128": input.add128(value as bigint);     break;
        case "uint256": input.add256(value as bigint);     break;
        case "address": input.addAddress(value as string); break;
        default: throw new Error(`Unsupported type in batch: ${type}`);
      }
    }

    return input.encrypt();
  });
}
```

### `templates/shared/lib/fhevm/decrypt.ts`

```typescript
import { getFhevmInstance } from "./instance";
import { wrap }             from "./errors";
import type { ChainKey }    from "./config";

// ─── Public Decryption ────────────────────────────────────────────────────────
//
// Public decryption is for values the CONTRACT wants to reveal globally.
// Examples: health factor check results, auction settlement prices, vote tallies.
//
// Flow:
//   1. Contract calls Gateway.requestDecryption() — async, takes 2-5 sec
//   2. Off-chain: call publicDecrypt(handles) here
//   3. Pass BOTH return values to the contract resolver (3 args total)
//
// CRITICAL BUG (FHEVM-007): Do NOT call fhe.getRelayer().publicDecrypt()
// getRelayer() does not exist on FhevmInstance.
// Call fhe.publicDecrypt() directly.

export interface PublicDecryptResult {
  abiEncodedClearValues: string;   // pass this to contract resolver
  decryptionProof:       string;   // pass this to contract resolver
  clearValues:           bigint[]; // parsed values for local use
}

/**
 * Request public decryption of one or more encrypted handles via the Relayer SDK.
 *
 * Returns both the raw SDK outputs needed by the contract resolver AND
 * the parsed cleartext values for local display/logic.
 *
 * @param handles Array of encrypted handles (use euint64.unwrap() in Solidity to get them)
 * @param chain   Target chain
 */
export async function publicDecrypt(
  handles: bigint[],
  chain:   ChainKey = "sepolia"
): Promise<PublicDecryptResult> {
  return wrap("publicDecrypt", async () => {
    const fhe = await getFhevmInstance(chain);

    // ✅ Correct: publicDecrypt lives directly on FhevmInstance
    // ❌ Wrong:  fhe.getRelayer().publicDecrypt() — FHEVM-007
    const result = await fhe.publicDecrypt(handles);

    const clearValues = parseAbiEncoded(result.abiEncodedClearValues, handles.length);

    return {
      abiEncodedClearValues: result.abiEncodedClearValues,
      decryptionProof:       result.decryptionProof,
      clearValues,
    };
  });
}

// ─── User Reencryption ────────────────────────────────────────────────────────
//
// Reencryption lets a wallet holder read their own encrypted value.
// The plaintext never travels over the network — only a re-encrypted
// version under an ephemeral key, which is decrypted locally.
//
// Requires a wallet signature (EIP-712) to prove ownership.

/**
 * Reencrypt a value so the owning wallet can read it.
 *
 * @param handle          Encrypted handle from the contract
 * @param contractAddress Contract storing the encrypted value
 * @param userAddress     Wallet address of the value owner
 * @param signer          ethers signer for EIP-712 signature
 */
export async function reencrypt(
  handle:          bigint,
  contractAddress: string,
  userAddress:     string,
  signer:          { signTypedData: (d: unknown, t: unknown, v: unknown) => Promise<string> },
  chain:           ChainKey = "sepolia"
): Promise<bigint> {
  return wrap("reencrypt", async () => {
    const fhe = await getFhevmInstance(chain);

    const { publicKey, privateKey } = fhe.generateKeypair();
    const eip712     = fhe.createEIP712(publicKey, contractAddress, userAddress);
    const signature  = await signer.signTypedData(eip712.domain, eip712.types, eip712.message);

    return fhe.reencrypt(
      handle, privateKey, publicKey, signature, contractAddress, userAddress
    );
  });
}

/**
 * Reencrypt multiple handles in one operation.
 * More efficient than calling reencrypt() separately for each handle.
 * All handles must belong to the same contract + user.
 */
export async function reencryptBatch(
  handles:         bigint[],
  contractAddress: string,
  userAddress:     string,
  signer:          { signTypedData: (d: unknown, t: unknown, v: unknown) => Promise<string> },
  chain:           ChainKey = "sepolia"
): Promise<bigint[]> {
  return wrap("reencryptBatch", async () => {
    const fhe = await getFhevmInstance(chain);

    const { publicKey, privateKey } = fhe.generateKeypair();
    const eip712    = fhe.createEIP712(publicKey, contractAddress, userAddress);
    const signature = await signer.signTypedData(eip712.domain, eip712.types, eip712.message);

    return Promise.all(
      handles.map((h) =>
        fhe.reencrypt(h, privateKey, publicKey, signature, contractAddress, userAddress)
      )
    );
  });
}

// Parse abiEncoded hex into array of bigint values (32 bytes each)
function parseAbiEncoded(encoded: string, count: number): bigint[] {
  const hex    = encoded.startsWith("0x") ? encoded.slice(2) : encoded;
  const values: bigint[] = [];
  for (let i = 0; i < count; i++) {
    values.push(BigInt("0x" + hex.slice(i * 64, (i + 1) * 64)));
  }
  return values;
}
```

### `templates/shared/lib/fhevm/gateway.ts`

```typescript
import { publicDecrypt } from "./decrypt";
import { FhevmError }    from "./errors";
import type { ChainKey } from "./config";

// ─── Gateway Callback Resolution ─────────────────────────────────────────────
//
// When a contract calls Gateway.requestDecryption(), the decryption result
// arrives as a callback transaction 2-5 seconds later on Sepolia.
//
// CRITICAL (FHEVM-009): The resolver contract function requires 3 arguments:
//   1. The identifying key (e.g. borrower address, auctionId)
//   2. result.abiEncodedClearValues  (from publicDecrypt)
//   3. result.decryptionProof        (from publicDecrypt)
//
// Passing only 1 arg (the key) will revert silently. Always pass all 3.

const POLL_INTERVAL_MS  = 2_000;
const DEFAULT_TIMEOUT_MS = 60_000;

/**
 * Poll for a contract event (use to detect when Gateway callback has fired).
 */
export async function pollForEvent(
  contract:    { on: Function; off: Function },
  eventName:   string,
  filter:      Record<string, string> = {},
  timeoutMs:   number = DEFAULT_TIMEOUT_MS
): Promise<unknown> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      contract.off(eventName, handler);
      reject(new FhevmError(
        `Event '${eventName}' not received within ${timeoutMs}ms. ` +
        `Verify the contract called Gateway.requestDecryption() and Sepolia gateway is reachable.`
      ));
    }, timeoutMs);

    const handler = (...args: unknown[]) => {
      const event = args[args.length - 1] as { args?: Record<string, string> };
      const matches = Object.entries(filter).every(
        ([k, v]) => event.args?.[k]?.toLowerCase() === v.toLowerCase()
      );
      if (!matches) return;

      clearTimeout(timer);
      contract.off(eventName, handler);
      resolve(event);
    };

    contract.on(eventName, handler);
  });
}

/**
 * Full health check resolve flow — 3-step pattern.
 *
 * Step 1: Get the pending health handle via getPendingHealthHandle()
 *         (NOT via getPositionHandles()[2] — only 2 values returned there)
 * Step 2: Call publicDecrypt() via Relayer SDK
 * Step 3: Call resolveHealthCheck(borrower, abiEncoded, proof) — 3 args, not 1
 *
 * @param vault    ethers Contract instance
 * @param borrower Address of the borrower position to resolve
 * @param signer   Wallet signer for submitting the resolve transaction
 */
export async function resolveHealthCheck(
  vault:    { getPendingHealthHandle: Function; resolveHealthCheck: Function; connect: Function },
  borrower: string,
  signer:   unknown,
  chain:    ChainKey = "sepolia"
): Promise<{ isUndercollateralized: boolean }> {
  // Step 1: Get pending handle — use dedicated getter, not getPositionHandles()
  const handle: bigint = await vault.getPendingHealthHandle(borrower);
  if (handle === 0n) {
    throw new FhevmError(
      `No pending health check for ${borrower}. ` +
      `Call requestHealthCheck(borrower) first.`
    );
  }

  // Step 2: Public decrypt
  const { abiEncodedClearValues, decryptionProof, clearValues } =
    await publicDecrypt([handle], chain);

  // Step 3: Resolve with 3 args (FHEVM-009 prevention)
  const tx = await (vault.connect(signer) as typeof vault).resolveHealthCheck(
    borrower,
    abiEncodedClearValues,
    decryptionProof
  );
  await (tx as { wait: Function }).wait();

  // clearValues[0]: 0n = healthy, 1n = undercollateralized
  return { isUndercollateralized: clearValues[0] === 1n };
}

/**
 * Full auction bid resolve flow. Same 3-step pattern as health check.
 */
export async function resolveAuctionBid(
  auction:   { getPendingBidHandle: Function; resolveBid: Function; connect: Function },
  auctionId: bigint,
  bidder:    string,
  signer:    unknown,
  chain:     ChainKey = "sepolia"
): Promise<{ bidWon: boolean; settlePrice: bigint }> {
  const handle: bigint = await auction.getPendingBidHandle(auctionId, bidder);
  if (handle === 0n) {
    throw new FhevmError(`No pending bid for ${bidder} on auction ${auctionId}`);
  }

  const { abiEncodedClearValues, decryptionProof, clearValues } =
    await publicDecrypt([handle], chain);

  const tx = await (auction.connect(signer) as typeof auction).resolveBid(
    auctionId,
    bidder,
    abiEncodedClearValues,
    decryptionProof
  );
  await (tx as { wait: Function }).wait();

  return { bidWon: clearValues[0] === 1n, settlePrice: clearValues[1] ?? 0n };
}
```

### `templates/shared/lib/fhevm/index.ts`

```typescript
// Central re-export for all FHEVM SDK helpers.
// Import from here in application code:
//   import { encryptUint64, publicDecrypt, reencryptBatch } from "@/lib/fhevm";

export { getFhevmInstance, resetFhevmInstance } from "./instance";
export type { ChainKey }                        from "./config";
export { CHAIN_CONFIGS }                        from "./config";
export { FhevmError, wrap }                     from "./errors";
export {
  encryptValue, encryptBatch,
  encryptBool, encryptUint8, encryptUint32,
  encryptUint64, encryptUint128, encryptAddress,
  type EncryptedInput,
}                                               from "./encrypt";
export {
  publicDecrypt, reencrypt, reencryptBatch,
  type PublicDecryptResult,
}                                               from "./decrypt";
export { resolveHealthCheck, resolveAuctionBid, pollForEvent } from "./gateway";
```

---

## Shared React Hooks

### `templates/shared/hooks/useEncrypt.ts`

```typescript
import { useState, useCallback }          from "react";
import { encryptValue, type EncryptedInput } from "../fhevm/encrypt";
import type { ChainKey }                  from "../fhevm/config";

export function useEncrypt(contractAddress: string, userAddress: string, chain?: ChainKey) {
  const [encrypting, setEncrypting] = useState(false);
  const [error, setError]           = useState<string | null>(null);

  const encrypt = useCallback(async (
    type:  "uint64" | "uint128" | "bool" | "address",
    value: bigint | boolean | string
  ): Promise<EncryptedInput | null> => {
    setEncrypting(true);
    setError(null);
    try {
      return await encryptValue(type, value, contractAddress, userAddress, chain);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Encryption failed");
      return null;
    } finally {
      setEncrypting(false);
    }
  }, [contractAddress, userAddress, chain]);

  return { encrypt, encrypting, error };
}
```

### `templates/shared/hooks/useReencrypt.ts`

```typescript
import { useState, useCallback } from "react";
import { reencryptBatch }        from "../fhevm/decrypt";
import type { ChainKey }         from "../fhevm/config";

export function useReencrypt(
  contractAddress: string,
  userAddress:     string,
  signer:          unknown,
  chain?:          ChainKey
) {
  const [values,  setValues]  = useState<bigint[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [error,   setError]   = useState<string | null>(null);

  const reveal = useCallback(async (handles: bigint[]) => {
    setLoading(true);
    setError(null);
    try {
      const decrypted = await reencryptBatch(handles, contractAddress, userAddress, signer as any, chain);
      setValues(decrypted);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Decryption failed");
    } finally {
      setLoading(false);
    }
  }, [contractAddress, userAddress, signer, chain]);

  return { values, reveal, loading, error };
}
```

### `templates/shared/hooks/useHealthCheck.ts`

```typescript
import { useState, useCallback }  from "react";
import { resolveHealthCheck }     from "../fhevm/gateway";
import type { ChainKey }          from "../fhevm/config";

type CheckStatus = "idle" | "requesting" | "waiting" | "resolving" | "done" | "error";

export function useHealthCheck(vault: unknown, signer: unknown, chain?: ChainKey) {
  const [status, setStatus] = useState<CheckStatus>("idle");
  const [result, setResult] = useState<boolean | null>(null);
  const [error,  setError]  = useState<string | null>(null);

  const check = useCallback(async (borrower: string) => {
    setStatus("requesting");
    setError(null);

    try {
      const v = vault as { connect: Function; requestHealthCheck: Function };

      // 1. Submit on-chain health check request
      const tx = await v.connect(signer).requestHealthCheck(borrower);
      await (tx as { wait: Function }).wait();

      // 2. Wait for Gateway callback (~2-5 sec on Sepolia)
      setStatus("waiting");
      await new Promise((r) => setTimeout(r, 4_000));

      // 3. Resolve (3-step: get handle → publicDecrypt → resolveHealthCheck with 3 args)
      setStatus("resolving");
      const { isUndercollateralized } = await resolveHealthCheck(v as any, borrower, signer, chain);

      setResult(isUndercollateralized);
      setStatus("done");
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Health check failed");
      setStatus("error");
    }
  }, [vault, signer, chain]);

  return { check, status, result, error };
}
```

---

## Shared Agent Runtime

### `templates/shared/agent/fhevm-agent.ts`

```typescript
import { Wallet, JsonRpcProvider, type Contract } from "ethers";
import { getFhevmInstance }    from "../lib/fhevm/instance";
import { encryptUint64, encryptBatch } from "../lib/fhevm/encrypt";
import { publicDecrypt }       from "../lib/fhevm/decrypt";
import { resolveHealthCheck, resolveAuctionBid } from "../lib/fhevm/gateway";
import type { ChainKey }       from "../lib/fhevm/config";

/**
 * Headless FHEVM runtime for autonomous agents.
 * Use this in monitor agents, bidder agents, and server-side scripts.
 *
 * Unlike the browser SDK, this does not require MetaMask or wallet injection.
 * It uses an ethers Wallet with a private key from environment variables.
 *
 * Usage:
 *   const agent = new FhevmAgent(process.env.SEPOLIA_RPC_URL!, process.env.AGENT_KEY!);
 *   const { handle, inputProof } = await agent.encryptUint64(2000n, vaultAddress);
 */
export class FhevmAgent {
  public  readonly wallet:   Wallet;
  private readonly provider: JsonRpcProvider;
  private readonly chain:    ChainKey;

  constructor(rpcUrl: string, privateKey: string, chain: ChainKey = "sepolia") {
    this.provider = new JsonRpcProvider(rpcUrl);
    this.wallet   = new Wallet(privateKey, this.provider);
    this.chain    = chain;
  }

  get address(): string {
    return this.wallet.address;
  }

  /** Encrypt a single uint64 value for the given contract */
  async encryptUint64(
    value:           bigint,
    contractAddress: string
  ): Promise<{ handle: Uint8Array; inputProof: Uint8Array }> {
    const result = await encryptUint64(value, contractAddress, this.wallet.address, this.chain);
    return { handle: result.handles[0], inputProof: result.inputProof };
  }

  /** Encrypt multiple values in one SDK call (single inputProof covers all) */
  async encryptBatch(
    inputs:          Array<{ type: string; value: bigint | boolean | string }>,
    contractAddress: string
  ) {
    return encryptBatch(inputs as any, contractAddress, this.wallet.address, this.chain);
  }

  /**
   * Public decrypt — gets abiEncodedClearValues + decryptionProof for resolver.
   * Pass BOTH return values to the contract's resolver function. (FHEVM-009)
   */
  async publicDecrypt(handles: bigint[]) {
    return publicDecrypt(handles, this.chain);
  }

  /**
   * Full health check resolve:
   *   1. getPendingHealthHandle(borrower)
   *   2. publicDecrypt([handle])
   *   3. resolveHealthCheck(borrower, abiEncoded, proof)
   */
  async resolveHealthCheck(vault: Contract, borrower: string) {
    return resolveHealthCheck(vault as any, borrower, this.wallet, this.chain);
  }

  /** Full auction bid resolve (same 3-step pattern) */
  async resolveAuctionBid(auction: Contract, auctionId: bigint, bidder: string) {
    return resolveAuctionBid(auction as any, auctionId, bidder, this.wallet, this.chain);
  }
}
```

---

## Lending Template — Solidity Contracts

### `templates/lending/src/ConfidentialVault.sol`

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import "@zama-ai/fhevm/contracts/lib/TFHE.sol";
import "@zama-ai/fhevm/contracts/gateway/GatewayInterface.sol";
import "./tokens/ConfidentialCollateral.sol";
import "./tokens/ConfidentialDebt.sol";

/**
 * @title ConfidentialVault
 * @notice Confidential lending vault using Zama FHEVM.
 *         Collateral and debt amounts are encrypted on-chain.
 *         Health factor checks run in FHE — no ZK proofs needed.
 *
 * Generated by fhevm-forge. See AGENT.md for development guidelines.
 */
contract ConfidentialVault is GatewayCallbackReceiver {

    // ─── State ───────────────────────────────────────────────────────────────

    ConfidentialCollateral public immutable collateralToken;
    ConfidentialDebt       public immutable debtToken;

    uint64 public constant LIQUIDATION_THRESHOLD = 150; // 150% collateralization required
    uint64 public constant TRIGGER_FEE_BPS       = 100; // 1% fee for liquidation trigger
    uint64 public constant PROTOCOL_FEE_BPS      = 50;  // 0.5% protocol fee

    struct Position {
        euint64 collateral; // encrypted collateral (gwei units)
        euint64 debt;       // encrypted debt (6-decimal USDC units)
        bool    active;
        uint256 openedAt;
    }

    mapping(address => Position) private _positions;
    address[]                    private _positionOwners;

    // Pending Gateway decryption requests
    mapping(uint256 => address) private _pendingHealthCheck;     // requestId => borrower
    mapping(uint256 => address) private _pendingRequestInitiator; // requestId => triggering agent
    mapping(address => euint64) private _pendingHealthHandle;    // borrower => handle

    // ─── Events ──────────────────────────────────────────────────────────────

    event PositionOpened(address indexed borrower, uint256 timestamp);
    event HealthCheckRequested(address indexed borrower, uint256 requestId);
    event AuctionTriggered(address indexed borrower, address indexed agent);
    event PositionClosed(address indexed borrower);

    // ─── Constructor ─────────────────────────────────────────────────────────

    constructor(address _collateral, address _debt) {
        collateralToken = ConfidentialCollateral(_collateral);
        debtToken       = ConfidentialDebt(_debt);
    }

    // ─── Lender Flow ─────────────────────────────────────────────────────────

    function depositLiquidity(einput encryptedAmount, bytes calldata inputProof) external {
        debtToken.depositLiquidity(encryptedAmount, inputProof);
    }

    // ─── Borrower Flow ───────────────────────────────────────────────────────

    function borrow(einput encryptedBorrowAmount, bytes calldata inputProof) external payable {
        require(msg.value > 0, "No ETH collateral");
        require(!_positions[msg.sender].active, "Position exists");

        // Wrap ETH into encrypted cETH
        collateralToken.wrapAndDeposit{value: msg.value}(msg.sender);
        euint64 collateral = collateralToken.encryptedBalanceOf(msg.sender);
        TFHE.allowThis(collateral);
        TFHE.allow(collateral, msg.sender);

        // Encrypt borrow amount
        euint64 debt = TFHE.asEuint64(encryptedBorrowAmount, inputProof);
        TFHE.allowThis(debt);
        TFHE.allow(debt, msg.sender);

        _positions[msg.sender] = Position({
            collateral: collateral,
            debt:       debt,
            active:     true,
            openedAt:   block.timestamp
        });
        _positionOwners.push(msg.sender);

        debtToken.mintDebt(msg.sender, debt);
        emit PositionOpened(msg.sender, block.timestamp);
    }

    // ─── Health Factor (FHE — replaces ZK circuits) ───────────────────────────

    function _computeHealthCheck(address borrower) internal returns (ebool) {
        Position storage pos = _positions[borrower];
        require(pos.active, "No active position");

        // Price is public (oracle) — used as a scalar multiplier
        uint64 ethPriceUsd = 3000_000000; // TODO: wire Chainlink oracle

        // Encrypted arithmetic — never reveals position size
        euint64 collateralUsd = TFHE.mul(pos.collateral, TFHE.asEuint64(ethPriceUsd));
        euint64 required      = TFHE.mul(pos.debt, TFHE.asEuint64(LIQUIDATION_THRESHOLD));
        euint64 scaled        = TFHE.mul(collateralUsd, TFHE.asEuint64(100));

        return TFHE.lt(scaled, required); // true = undercollateralized
    }

    // ─── Liquidation Flow ────────────────────────────────────────────────────

    function requestHealthCheck(address borrower) external returns (uint256 requestId) {
        ebool unhealthy = _computeHealthCheck(borrower);

        uint256[] memory cts = new uint256[](1);
        cts[0] = Gateway.toUint256(unhealthy);

        requestId = Gateway.requestDecryption(
            cts,
            this._onHealthCheckDecrypted.selector,
            0,
            false
        );

        _pendingHealthCheck[requestId]      = borrower;
        _pendingRequestInitiator[requestId] = tx.origin;

        emit HealthCheckRequested(borrower, requestId);
    }

    /// @notice Zama Gateway callback — fires ~2-5 sec after requestHealthCheck()
    /// @dev onlyGateway prevents anyone else from calling this with fake results
    function _onHealthCheckDecrypted(
        uint256 requestId,
        bool    isUndercollateralized
    ) external onlyGateway {
        address borrower        = _pendingHealthCheck[requestId];
        address triggeringAgent = _pendingRequestInitiator[requestId];
        delete _pendingHealthCheck[requestId];
        delete _pendingRequestInitiator[requestId];

        if (!isUndercollateralized) return;

        _positions[borrower].active = false;
        emit AuctionTriggered(borrower, triggeringAgent);
        // TODO: call DutchAuction.startAuction(borrower, position.collateral, position.debt)
    }

    // ─── Getters ─────────────────────────────────────────────────────────────

    /// @notice Returns collateral and debt handles for a position.
    ///         Only 2 values. Health handle is separate — use getPendingHealthHandle().
    function getPositionHandles(address borrower)
        external view
        returns (uint256 collateralHandle, uint256 debtHandle)
    {
        Position storage pos = _positions[borrower];
        return (euint64.unwrap(pos.collateral), euint64.unwrap(pos.debt));
    }

    /// @notice Returns the pending health check handle for Relayer SDK resolution.
    ///         Returns bytes32(0) if no check is pending.
    function getPendingHealthHandle(address borrower) external view returns (bytes32) {
        euint64 handle = _pendingHealthHandle[borrower];
        return TFHE.isInitialized(handle) ? TFHE.toBytes32(handle) : bytes32(0);
    }

    /// @notice Returns true if a health check is pending for this borrower.
    function hasPendingCheck(address borrower) external view returns (bool) {
        return TFHE.isInitialized(_pendingHealthHandle[borrower]);
    }

    /// @notice Returns named loan info. Status is a separate hasPendingCheck() call.
    function getLoanInfo(address borrower)
        external view
        returns (
            bool    active,
            uint256 openedAt
        )
    {
        Position storage pos = _positions[borrower];
        return (pos.active, pos.openedAt);
    }

    function getActivePositions() external view returns (address[] memory) {
        uint256 count;
        for (uint256 i; i < _positionOwners.length; ++i) {
            if (_positions[_positionOwners[i]].active) ++count;
        }
        address[] memory active = new address[](count);
        uint256 j;
        for (uint256 i; i < _positionOwners.length; ++i) {
            if (_positions[_positionOwners[i]].active) active[j++] = _positionOwners[i];
        }
        return active;
    }

    // ─── Access Control ───────────────────────────────────────────────────────

    /// @notice Grant an external agent FHE read access to a position.
    ///         Called by x402 server after payment is verified.
    function grantAgentAccess(address borrower, address agent) external onlyOwner {
        Position storage pos = _positions[borrower];
        require(pos.active, "Position not active");
        TFHE.allow(pos.collateral, agent);
        TFHE.allow(pos.debt,       agent);
    }
}
```

### `templates/lending/test/ConfidentialVault.t.sol`

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import { Test }           from "forge-std/Test.sol";
import { FHEVMTestBase } from "forge-fhevm/FHEVMTestBase.sol";
import { ConfidentialVault }      from "../src/ConfidentialVault.sol";
import { ConfidentialCollateral } from "../src/tokens/ConfidentialCollateral.sol";
import { ConfidentialDebt }       from "../src/tokens/ConfidentialDebt.sol";
import { TFHE }                   from "fhevm/lib/TFHE.sol";

contract ConfidentialVaultTest is FHEVMTestBase {
    ConfidentialVault      vault;
    ConfidentialCollateral collateral;
    ConfidentialDebt       debt;

    address borrower = makeAddr("borrower");
    address lender   = makeAddr("lender");
    address agent    = makeAddr("agent");

    function setUp() public override {
        super.setUp(); // deploys all forge-fhevm host contracts

        collateral = new ConfidentialCollateral();
        debt       = new ConfidentialDebt();
        vault      = new ConfidentialVault(address(collateral), address(debt));

        collateral.setVault(address(vault));
        debt.setVault(address(vault));
    }

    function test_borrow_opens_position() public {
        uint64 borrowAmountUsdc = 2000_000000; // $2000

        // Encrypt borrow amount using forge-fhevm test helper
        (bytes32 handle, bytes memory inputProof) =
            encryptUint64(borrowAmountUsdc, address(vault), borrower);

        vm.prank(borrower);
        vm.deal(borrower, 1.5 ether);
        vault.borrow{value: 1.5 ether}(einput.wrap(handle), inputProof);

        address[] memory positions = vault.getActivePositions();
        assertEq(positions.length, 1);
        assertEq(positions[0], borrower);
    }

    function test_get_position_handles_returns_exactly_two_values() public {
        // Borrow first
        test_borrow_opens_position();

        (uint256 collHandle, uint256 debtHandle) = vault.getPositionHandles(borrower);
        assertTrue(collHandle != 0, "Collateral handle should be non-zero");
        assertTrue(debtHandle  != 0, "Debt handle should be non-zero");
        // There is no third handle here — health handle is separate
    }

    function test_has_no_pending_check_initially() public {
        test_borrow_opens_position();
        assertFalse(vault.hasPendingCheck(borrower));
    }

    function test_health_check_request_triggers_gateway() public {
        test_borrow_opens_position();

        vm.prank(agent);
        uint256 requestId = vault.requestHealthCheck(borrower);
        assertTrue(requestId > 0);
        // forge-fhevm resolves Gateway callbacks synchronously in tests
    }

    function test_grant_agent_access() public {
        test_borrow_opens_position();

        vault.grantAgentAccess(borrower, agent);
        // After this, the agent can call fhevmjs.reencrypt() to read the position
        // Verify by checking the ACL (forge-fhevm exposes this)
        assertTrue(TFHE.isAllowed(vault.getPositionHandles(borrower).collateralHandle, agent));
    }
}
```

---

## Blank Template

### `templates/blank/src/Counter.sol`

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import { TFHE }               from "fhevm/lib/TFHE.sol";
import { SepoliaZamaFHEVMConfig } from "fhevm/config/ZamaFHEVMConfig.sol";

/**
 * @title Counter — Minimal FHEVM Example
 * @notice A confidential counter where the value is encrypted on-chain.
 *         Generated by fhevm-forge. See AGENT.md for development guidelines.
 */
contract Counter is SepoliaZamaFHEVMConfig {
    euint64 private _count;

    constructor() {
        _count = TFHE.asEuint64(0);
        TFHE.allowThis(_count); // contract can use this handle in future txs
    }

    /// @notice Add an encrypted value to the counter
    function add(einput encryptedAmount, bytes calldata inputProof) external {
        euint64 amount = TFHE.asEuint64(encryptedAmount, inputProof);
        _count = TFHE.add(_count, amount);
        TFHE.allowThis(_count);          // required after every assignment
        TFHE.allow(_count, msg.sender);  // sender can reencrypt and read
    }

    /// @notice Returns the encrypted handle — use fhevmjs.reencrypt() to read
    function getHandle() external view returns (uint256) {
        return euint64.unwrap(_count);
    }
}
```

### `templates/blank/test/Counter.t.sol`

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import { Test }          from "forge-std/Test.sol";
import { FHEVMTestBase } from "forge-fhevm/FHEVMTestBase.sol";
import { Counter }       from "../src/Counter.sol";

contract CounterTest is FHEVMTestBase {
    Counter counter;

    function setUp() public override {
        super.setUp();
        counter = new Counter();
    }

    function test_add_increases_encrypted_count() public {
        address user = makeAddr("user");
        (bytes32 handle, bytes memory proof) =
            encryptUint64(10, address(counter), user);

        vm.prank(user);
        counter.add(einput.wrap(handle), proof);

        uint256 resultHandle = counter.getHandle();
        // Decrypt using forge-fhevm test helper (only works in tests)
        uint64 decrypted = decryptUint64(euint64.wrap(bytes32(resultHandle)));
        assertEq(decrypted, 10);
    }
}
```
