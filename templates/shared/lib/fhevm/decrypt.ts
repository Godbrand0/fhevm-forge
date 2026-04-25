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
    const eip712    = fhe.createEIP712(publicKey, contractAddress, userAddress);
    const signature = await signer.signTypedData(eip712.domain, eip712.types, eip712.message);

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
