// Central re-export for all FHEVM SDK helpers.
// Import from here in application code:
//   import { encryptUint64, publicDecrypt } from "@/lib/fhevm";

export { getFhevmInstance, resetFhevmInstance } from "./instance";
export type { ChainKey, FhevmChainConfig }       from "./config";
export { CHAIN_CONFIGS }                         from "./config";
export { FhevmError, wrap }                      from "./errors";
export {
  encryptValue, encryptBatch,
  encryptBool, encryptUint8, encryptUint32,
  encryptUint64, encryptUint128, encryptAddress,
  type EncryptedInput,
}                                                from "./encrypt";
export {
  publicDecrypt, reencrypt, reencryptBatch,
  type PublicDecryptResult,
}                                                from "./decrypt";
export { resolveHealthCheck, resolveAuctionBid, pollForEvent } from "./gateway";
