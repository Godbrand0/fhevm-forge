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
