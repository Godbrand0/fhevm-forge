import { Wallet, JsonRpcProvider, type Contract } from "ethers";
import { getFhevmInstance }                       from "../lib/fhevm/instance";
import { encryptUint64, encryptBatch }            from "../lib/fhevm/encrypt";
import { publicDecrypt }                          from "../lib/fhevm/decrypt";
import { resolveHealthCheck, resolveAuctionBid }  from "../lib/fhevm/gateway";
import type { ChainKey }                          from "../lib/fhevm/config";

/**
 * Headless FHEVM runtime for autonomous agents.
 * Use this in monitor agents, bidder agents, and server-side scripts.
 *
 * Unlike the browser SDK, this does not require MetaMask or wallet injection.
 * It uses an ethers Wallet with a private key from environment variables.
 *
 * Usage:
 *   const agent = new FhevmAgent(process.env.SEPOLIA_RPC_URL!, process.env.AGENT_KEY!);
 *   const { handle, inputProof } = await agent.encryptUint64(2000n, vaultAddress);
 */
export class FhevmAgent {
  public  readonly wallet:   Wallet;
  private readonly provider: JsonRpcProvider;
  private readonly chain:    ChainKey;

  constructor(rpcUrl: string, privateKey: string, chain: ChainKey = "sepolia") {
    this.provider = new JsonRpcProvider(rpcUrl);
    this.wallet   = new Wallet(privateKey, this.provider);
    this.chain    = chain;
  }

  get address(): string {
    return this.wallet.address;
  }

  /** Encrypt a single uint64 value for the given contract */
  async encryptUint64(
    value:           bigint,
    contractAddress: string
  ): Promise<{ handle: Uint8Array; inputProof: Uint8Array }> {
    const result = await encryptUint64(value, contractAddress, this.wallet.address, this.chain);
    return { handle: result.handles[0], inputProof: result.inputProof };
  }

  /** Encrypt multiple values in one SDK call (single inputProof covers all) */
  async encryptBatch(
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    inputs:          Array<{ type: string; value: bigint | boolean | string }>,
    contractAddress: string
  ) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return encryptBatch(inputs as any, contractAddress, this.wallet.address, this.chain);
  }

  /**
   * Public decrypt — gets abiEncodedClearValues + decryptionProof for resolver.
   * Pass BOTH return values to the contract's resolver function. (FHEVM-009)
   */
  async publicDecrypt(handles: bigint[]) {
    return publicDecrypt(handles, this.chain);
  }

  /**
   * Full health check resolve:
   *   1. getPendingHealthHandle(borrower)
   *   2. publicDecrypt([handle])
   *   3. resolveHealthCheck(borrower, abiEncoded, proof)
   */
  async resolveHealthCheck(vault: Contract, borrower: string) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return resolveHealthCheck(vault as any, borrower, this.wallet, this.chain);
  }

  /** Full auction bid resolve (same 3-step pattern) */
  async resolveAuctionBid(auction: Contract, auctionId: bigint, bidder: string) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return resolveAuctionBid(auction as any, auctionId, bidder, this.wallet, this.chain);
  }
}
