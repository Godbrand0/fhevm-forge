import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Required so Next.js can bundle the FHEVM WASM module
  webpack(config) {
    config.experiments = { ...config.experiments, asyncWebAssembly: true };
    return config;
  },
};

export default nextConfig;
