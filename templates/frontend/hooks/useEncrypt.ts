"use client";
import { useState, useCallback }               from "react";
import { useAccount }                          from "wagmi";
import { useFhevm }                            from "@fhevm/sdk";
import { encryptValue, type EncryptedInput }   from "@fhevm/sdk";

/**
 * Wagmi-compatible encrypt hook.
 */
export function useEncrypt(contractAddress: string) {
  const { address: userAddress }    = useAccount();
  const [encrypting, setEncrypting] = useState(false);
  const [error, setError]           = useState<string | null>(null);
  const { instance }                = useFhevm();

  const encrypt = useCallback(async (
    type:  "uint64" | "uint128" | "bool" | "address",
    value: bigint | boolean | string
  ): Promise<EncryptedInput | null> => {
    if (!userAddress) { setError("Wallet not connected"); return null; }
    if (!instance)    { setError("FHEVM not initialized"); return null; }

    setEncrypting(true);
    setError(null);
    try {
      return await encryptValue(type, value, contractAddress, userAddress, instance);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Encryption failed");
      return null;
    } finally {
      setEncrypting(false);
    }
  }, [contractAddress, userAddress, instance]);

  return { encrypt, encrypting, error };
}
