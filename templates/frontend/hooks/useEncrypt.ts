"use client";
import { useState, useCallback }               from "react";
import { useAccount }                          from "wagmi";
import { encryptValue, type EncryptedInput }   from "@/lib/fhevm/encrypt";
import type { ChainKey }                       from "@/lib/fhevm/config";

/**
 * Wagmi-compatible encrypt hook.
 * Reads userAddress from the connected wallet automatically.
 */
export function useEncrypt(contractAddress: string, chain?: ChainKey) {
  const { address: userAddress }    = useAccount();
  const [encrypting, setEncrypting] = useState(false);
  const [error, setError]           = useState<string | null>(null);

  const encrypt = useCallback(async (
    type:  "uint64" | "uint128" | "bool" | "address",
    value: bigint | boolean | string
  ): Promise<EncryptedInput | null> => {
    if (!userAddress) { setError("Wallet not connected"); return null; }
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
