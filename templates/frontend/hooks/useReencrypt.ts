"use client";
import { useState, useCallback }  from "react";
import { useSignTypedData }        from "wagmi";
import { useFhevm }                from "@fhevm/sdk";
import { reencryptBatch }          from "@fhevm/sdk";

/**
 * Wagmi-compatible reencrypt hook.
 * Uses one signature for multiple handles via the Relayer SDK's userDecrypt.
 */
export function useReencrypt(
  contractAddress: string,
  userAddress:     string
) {
  const [values,  setValues]  = useState<bigint[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [error,   setError]   = useState<string | null>(null);
  const { instance }          = useFhevm();

  const { signTypedDataAsync } = useSignTypedData();

  const reveal = useCallback(async (handles: bigint[]) => {
    if (!userAddress) { setError("Wallet not connected"); return; }
    if (!instance)    { setError("FHEVM not initialized"); return; }

    setLoading(true);
    setError(null);

    try {
      const handleStrings = handles.map(h => "0x" + h.toString(16));

      const decrypted = await reencryptBatch(
        handleStrings,
        contractAddress,
        userAddress,
        {
          // Adapt wagmi signer to the SDK expected interface
          signTypedData: async (domain: any, types: any, message: any) => {
            return signTypedDataAsync({
              domain,
              types,
              primaryType: Object.keys(types)[0],
              message
            });
          }
        },
        instance
      );

      setValues(decrypted);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Decryption failed");
    } finally {
      setLoading(false);
    }
  }, [contractAddress, userAddress, instance, signTypedDataAsync]);

  return { values, reveal, loading, error };
}
