import { useState, useCallback } from "react";
import { resolveHealthCheck }    from "../fhevm/gateway";
import type { ChainKey }         from "../fhevm/config";

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

      const tx = await v.connect(signer).requestHealthCheck(borrower);
      await (tx as { wait: Function }).wait();

      setStatus("waiting");
      await new Promise((r) => setTimeout(r, 4_000));

      setStatus("resolving");
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
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
