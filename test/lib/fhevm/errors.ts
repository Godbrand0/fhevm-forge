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
