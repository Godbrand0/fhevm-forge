"use client";
import { useState }                                             from "react";
import { useAccount, useConnect, useDisconnect,
         useWriteContract, useReadContract }                    from "wagmi";
import { injected }                                             from "wagmi/connectors";
import { useFhevm }                                              from "@fhevm/sdk";
import { useEncrypt }                                            from "../hooks/useEncrypt";
import { useReencrypt }                                          from "../hooks/useReencrypt";
import { CONTRACT_ADDRESS, COUNTER_ABI }                        from "./contract";

export default function Page() {
  const { address, isConnected }           = useAccount();
  const { connect }                        = useConnect();
  const { disconnect }                     = useDisconnect();
  const { writeContractAsync, isPending }  = useWriteContract();

  const { loading: fheLoading, error: fheError } = useFhevm();

  const { encrypt, encrypting, error: encryptError } =
    useEncrypt(CONTRACT_ADDRESS);

  const { values, reveal, loading: revealing, error: revealError } =
    useReencrypt(CONTRACT_ADDRESS, address ?? "");

  const [numberInput, setNumberInput] = useState("");
  const [txStatus,    setTxStatus]    = useState<string | null>(null);

  // Read the encrypted handle stored in the contract
  const { data: handle, refetch } = useReadContract({
    address:      CONTRACT_ADDRESS as `0x${string}`,
    abi:          COUNTER_ABI,
    functionName: "getHandle",
    query:        { enabled: isConnected },
  });

  async function handleIncrement() {
    setTxStatus(null);
    try {
      const hash = await writeContractAsync({
        address:      CONTRACT_ADDRESS as `0x${string}`,
        abi:          COUNTER_ABI,
        functionName: "increment",
      });
      setTxStatus(`Tx sent: ${hash}`);
      await refetch();
    } catch (err: unknown) {
      setTxStatus(err instanceof Error ? err.message : "Transaction failed");
    }
  }

  async function handleSetNumber() {
    if (!numberInput || !address) return;
    setTxStatus(null);
    const encrypted = await encrypt("uint64", BigInt(numberInput));
    if (!encrypted) return;
    try {
      // handles[0] is the ciphertext handle (bytes32); inputProof is the ZK proof
      const hash = await writeContractAsync({
        address:      CONTRACT_ADDRESS as `0x${string}`,
        abi:          COUNTER_ABI,
        functionName: "setNumber",
        args: [
          `0x${Buffer.from(encrypted.handles[0]).toString("hex")}` as `0x${string}`,
          `0x${Buffer.from(encrypted.inputProof).toString("hex")}` as `0x${string}`,
        ],
      });
      setTxStatus(`Tx sent: ${hash}`);
      await refetch();
    } catch (err: unknown) {
      setTxStatus(err instanceof Error ? err.message : "Transaction failed");
    }
  }

  // Reencrypt: wallet signs an EIP-712 message; the relayer SDK decrypts locally
  async function handleReveal() {
    if (!handle) return;
    await reveal([BigInt(handle as string)]);
  }

  if (fheLoading) {
    return (
      <main>
        <h1>Initializing FHEVM…</h1>
        <p>Loading WebAssembly modules and secure context.</p>
      </main>
    );
  }

  if (fheError) {
    return (
      <main>
        <h1>FHEVM Error</h1>
        <p className="error">{fheError}</p>
        <button onClick={() => window.location.reload()}>Retry</button>
      </main>
    );
  }

  if (!isConnected) {
    return (
      <main>
        <h1>Counter — FHEVM</h1>
        <p>Encrypted counter powered by Zama FHEVM.<br />
           The value is never revealed on-chain.</p>
        <button onClick={() => connect({ connector: injected() })}>
          Connect Wallet
        </button>
      </main>
    );
  }

  return (
    <main>
      <h1>Counter — FHEVM</h1>
      <p><span className="badge">{address}</span></p>
      <button className="secondary" onClick={() => disconnect()}>Disconnect</button>

      <hr />

      {/* ── Read encrypted value ───────────────────────────────────────────── */}
      <h2>Encrypted Value</h2>
      <p>Handle (bytes32 on-chain):</p>
      <p><span className="badge">{handle ? String(handle) : "—"}</span></p>

      <button onClick={handleReveal} disabled={revealing || !handle}>
        {revealing ? "Decrypting…" : "Reveal My Value"}
      </button>
      <p style={{ fontSize: "0.75rem", color: "#6e7681" }}>
        Signs an EIP-712 message with your wallet; the relayer SDK decrypts locally.
        Your plaintext never leaves the browser.
      </p>

      {values !== null && (
        <p className="value">Decrypted: {String(values[0])}</p>
      )}
      {revealError && <p className="error">{revealError}</p>}

      <hr />

      {/* ── Increment ─────────────────────────────────────────────────────── */}
      <h2>Increment</h2>
      <p>Adds 1 to the encrypted counter. No encryption needed — 1 is a public constant.</p>
      <button onClick={handleIncrement} disabled={isPending}>
        {isPending ? "Sending…" : "Increment by 1"}
      </button>

      <hr />

      {/* ── Set number ────────────────────────────────────────────────────── */}
      <h2>Set Number</h2>
      <p>
        Encrypts your value locally via the Relayer SDK, then submits the ciphertext
        + input proof to the contract. The contract never sees the plaintext.
      </p>
      <input
        type="number"
        min="0"
        value={numberInput}
        onChange={(e) => setNumberInput(e.target.value)}
        placeholder="Value (uint64)"
      />
      <button onClick={handleSetNumber} disabled={encrypting || isPending || !numberInput}>
        {encrypting ? "Encrypting…" : "Set Number"}
      </button>
      {encryptError && <p className="error">{encryptError}</p>}

      {txStatus && <p className="status">{txStatus}</p>}
    </main>
  );
}
