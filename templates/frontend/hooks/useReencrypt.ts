"use client";
import { useState, useCallback }  from "react";
import { useSignTypedData }        from "wagmi";
import { getFhevmInstance }        from "@/lib/fhevm/instance";
import { wrap }                    from "@/lib/fhevm/errors";
import type { ChainKey }           from "@/lib/fhevm/config";

/**
 * Wagmi-compatible reencrypt hook.
 *
 * Uses wagmi's useSignTypedData instead of an ethers signer so it works
 * with any injected wallet (MetaMask, Rabby, etc.) out of the box.
 *
 * Flow:
 *   1. Generate a fresh ephemeral keypair
 *   2. Build an EIP-712 payload that proves wallet ownership
 *   3. Prompt the wallet to sign (no on-chain tx, no gas)
 *   4. Decrypt locally — plaintext never travels over the network
 */
export function useReencrypt(
  contractAddress: string,
  userAddress:     string,
  chain?:          ChainKey
) {
  const [values,  setValues]  = useState<bigint[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [error,   setError]   = useState<string | null>(null);

  const { signTypedDataAsync } = useSignTypedData();

  const reveal = useCallback(async (handles: bigint[]) => {
    if (!userAddress) { setError("Wallet not connected"); return; }
    setLoading(true);
    setError(null);

    try {
      const results = await wrap("reencrypt", async () => {
        const fhe = await getFhevmInstance(chain ?? "sepolia");
        const { publicKey, privateKey } = fhe.generateKeypair();

        return Promise.all(handles.map(async (handle) => {
          const eip712 = fhe.createEIP712(publicKey, contractAddress, userAddress);

          // Wallet signs — user sees a human-readable "Decrypt my value" prompt
          const signature = await signTypedDataAsync({
            domain:      eip712.domain      as Parameters<typeof signTypedDataAsync>[0]["domain"],
            types:       eip712.types       as Parameters<typeof signTypedDataAsync>[0]["types"],
            primaryType: Object.keys(eip712.types)[0] as string,
            message:     eip712.message     as Parameters<typeof signTypedDataAsync>[0]["message"],
          });

          return fhe.reencrypt(
            handle, privateKey, publicKey, signature, contractAddress, userAddress
          );
        }));
      });

      setValues(results);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Decryption failed");
    } finally {
      setLoading(false);
    }
  }, [contractAddress, userAddress, chain, signTypedDataAsync]);

  return { values, reveal, loading, error };
}
