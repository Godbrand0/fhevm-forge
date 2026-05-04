import { getFhevmInstance } from "./instance";
import { wrap }             from "./errors";
import type { ChainKey }    from "./config";

export interface PublicDecryptResult {
  abiEncodedClearValues: string;
  decryptionProof:       string;
  clearValues:           bigint[];
}

/**
 * Request public decryption of one or more encrypted handles via the Relayer SDK.
 *
 * CRITICAL: Do NOT call fhe.getRelayer().publicDecrypt() — getRelayer() does not exist.
 * Call fhe.publicDecrypt() directly on the FhevmInstance.
 */
export async function publicDecrypt(
  handles: bigint[],
  chain:   ChainKey = "sepolia"
): Promise<PublicDecryptResult> {
  return wrap("publicDecrypt", async () => {
    const fhe    = await getFhevmInstance(chain);
    const result = await fhe.publicDecrypt(handles);
    return {
      abiEncodedClearValues: result.abiEncodedClearValues,
      decryptionProof:       result.decryptionProof,
      clearValues:           parseAbiEncoded(result.abiEncodedClearValues, handles.length),
    };
  });
}

/**
 * Reencrypt a value so the owning wallet can read it.
 * Requires an EIP-712 wallet signature to prove ownership.
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
    return fhe.reencrypt(handle, privateKey, publicKey, signature, contractAddress, userAddress);
  });
}

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

function parseAbiEncoded(encoded: string, count: number): bigint[] {
  const hex    = encoded.startsWith("0x") ? encoded.slice(2) : encoded;
  const values: bigint[] = [];
  for (let i = 0; i < count; i++) {
    values.push(BigInt("0x" + hex.slice(i * 64, (i + 1) * 64)));
  }
  return values;
}
