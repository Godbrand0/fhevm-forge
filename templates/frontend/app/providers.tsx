"use client";
import { WagmiProvider, createConfig, http } from "wagmi";
import { sepolia }                           from "wagmi/chains";
import { injected }                          from "wagmi/connectors";
import { QueryClient, QueryClientProvider }  from "@tanstack/react-query";
import { FhevmProvider }                     from "@fhevm/sdk";

const wagmiConfig = createConfig({
  chains:      [sepolia],
  connectors:  [injected()],
  transports:  { [sepolia.id]: http(process.env.NEXT_PUBLIC_SEPOLIA_RPC_URL) },
});

const queryClient = new QueryClient();

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <WagmiProvider config={wagmiConfig}>
      <QueryClientProvider client={queryClient}>
        <FhevmProvider>
          {children}
        </FhevmProvider>
      </QueryClientProvider>
    </WagmiProvider>
  );
}
