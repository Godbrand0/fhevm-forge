"use client";

import { motion } from "framer-motion";
import { CodeBlock } from "@/components/ui/CodeBlock";
import { FeatureCard } from "@/components/ui/FeatureCard";
import {
  Terminal,
  Zap,
  ShieldCheck,
  Bot,
  Box,
  Layers,
  ArrowRight,
  Code2,
  Cpu,
} from "lucide-react";
import Link from "next/link";

export default function Home() {
  return (
    <div className="relative min-h-screen bg-surface selection:bg-brand-500/30">
      {/* Background Grid */}
      <div className="pointer-events-none fixed inset-0 z-0 bg-grid-white opacity-[0.03] mask-image-fade" />
      <div className="pointer-events-none fixed inset-0 z-0 bg-dot-white opacity-[0.03] mask-image-fade" />

      {/* Navigation */}
      <nav className="relative z-50 border-b border-white/5 bg-surface/50 backdrop-blur-xl">
        <div className="mx-auto flex h-16 max-w-7xl items-center justify-between px-6">
          <div className="flex items-center gap-2">
            <Box className="h-6 w-6 text-brand-500" />
            <span className="text-lg font-bold tracking-tight text-white">fhevm-forge</span>
          </div>
          <div className="flex items-center gap-6 text-sm font-medium text-text-muted">
            <Link href="https://crates.io/crates/fhevm-forge" target="_blank" className="hover:text-white transition-colors">
              Crates.io
            </Link>
            <Link href="https://www.npmjs.com/package/fhevm-forge-sdk" target="_blank" className="hover:text-white transition-colors">
              NPM
            </Link>
            <Link href="https://github.com/Godbrand0/fhevm-forge" target="_blank" className="hover:text-white transition-colors">
              GitHub
            </Link>
          </div>
        </div>
      </nav>

      <main className="relative z-10 mx-auto max-w-7xl px-6 pt-24 pb-32">
        {/* Hero Section */}
        <section className="flex flex-col items-center justify-center pt-16 pb-32 text-center">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5 }}
            className="inline-flex items-center gap-2 rounded-full border border-brand-500/30 bg-brand-500/10 px-4 py-1.5 text-sm font-medium text-brand-500"
          >
            <Zap className="h-4 w-4" />
            <span>Foundry scaffold for Zama FHEVM</span>
          </motion.div>

          <motion.h1
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="mt-8 max-w-4xl text-5xl font-extrabold tracking-tight text-white sm:text-7xl"
          >
            Build Confidential Smart Contracts,{" "}
            <span className="text-transparent bg-clip-text bg-gradient-to-r from-brand-500 to-cyan-400">
              Faster.
            </span>
          </motion.h1>

          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.2 }}
            className="mt-6 max-w-2xl text-lg text-text-muted sm:text-xl"
          >
            The ultimate CLI tool and scaffold to build, test, and deploy confidential applications on Zama FHEVM. Less boilerplate, zero context switching.
          </motion.p>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: 0.3 }}
            className="mt-10 flex flex-col items-center gap-4 sm:flex-row"
          >
            <Link
              href="https://crates.io/crates/fhevm-forge"
              target="_blank"
              className="group flex h-12 items-center gap-2 rounded-lg bg-white px-6 font-semibold text-surface transition-all hover:bg-white/90"
            >
              Get Started
              <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-1" />
            </Link>
            <CodeBlock code="cargo install fhevm-forge" className="h-12 min-w-[280px]" />
          </motion.div>
        </section>

        {/* Workflow Section */}
        <section className="py-24">
          <div className="mb-16 text-center">
            <h2 className="text-3xl font-bold text-white sm:text-4xl">How it works</h2>
            <p className="mt-4 text-text-muted">A streamlined path from idea to mainnet.</p>
          </div>

          <div className="grid gap-8 md:grid-cols-3">
            <div className="relative flex flex-col items-center text-center">
              <div className="mb-6 flex h-16 w-16 items-center justify-center rounded-2xl border border-white/10 bg-surface-bright text-white shadow-xl">
                <Terminal className="h-8 w-8" />
              </div>
              <h3 className="mb-2 text-xl font-semibold text-white">1. Scaffold</h3>
              <p className="text-sm text-text-muted">
                Run <code className="text-brand-500">fhevm-forge init</code> to generate a full Foundry project with a React Next.js frontend and local mock FHE environment.
              </p>
            </div>
            <div className="relative flex flex-col items-center text-center">
              <div className="mb-6 flex h-16 w-16 items-center justify-center rounded-2xl border border-white/10 bg-surface-bright text-white shadow-xl">
                <ShieldCheck className="h-8 w-8" />
              </div>
              <h3 className="mb-2 text-xl font-semibold text-white">2. Develop & Lint</h3>
              <p className="text-sm text-text-muted">
                Write your logic using TFHE types. Use <code className="text-brand-500">fhevm-forge lint</code> to catch silent bugs before they happen.
              </p>
            </div>
            <div className="relative flex flex-col items-center text-center">
              <div className="mb-6 flex h-16 w-16 items-center justify-center rounded-2xl border border-white/10 bg-surface-bright text-white shadow-xl">
                <Layers className="h-8 w-8" />
              </div>
              <h3 className="mb-2 text-xl font-semibold text-white">3. Deploy</h3>
              <p className="text-sm text-text-muted">
                Deploy across multiple networks instantly with <code className="text-brand-500">fhevm-forge deploy --chains sepolia</code>.
              </p>
            </div>
          </div>
        </section>

        {/* Agent-Native Section */}
        <section className="relative overflow-hidden rounded-3xl border border-brand-500/20 bg-brand-500/5 px-8 py-24 sm:px-16 mt-12">
          <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,rgba(59,130,246,0.15),transparent)]" />
          <div className="relative z-10 mx-auto grid max-w-5xl items-center gap-16 lg:grid-cols-2">
            <div>
              <div className="mb-6 inline-flex h-12 w-12 items-center justify-center rounded-xl bg-brand-500/20 text-brand-500 ring-1 ring-brand-500/30">
                <Bot className="h-6 w-6" />
              </div>
              <h2 className="mb-6 text-3xl font-bold tracking-tight text-white sm:text-4xl">
                Agent-Native by Design
              </h2>
              <p className="mb-6 text-lg text-text-muted">
                Every scaffolded project includes a specialized <code className="text-white bg-surface-bright px-1.5 py-0.5 rounded border border-white/10">AGENT.md</code> file. This acts as a highly specialized "skill" for AI coding assistants like Claude, Antigravity, or ChatGPT.
              </p>
              <ul className="space-y-4 text-text-muted">
                <li className="flex items-start gap-3">
                  <CheckCircle className="mt-1 h-5 w-5 shrink-0 text-brand-500" />
                  <span><strong>Zero-Shot Context:</strong> Models instantly understand FHEVM 0.11+ APIs without hallucinations.</span>
                </li>
                <li className="flex items-start gap-3">
                  <CheckCircle className="mt-1 h-5 w-5 shrink-0 text-brand-500" />
                  <span><strong>No Docs Hunting:</strong> Stop bouncing between Zama documentation and your editor. Everything the agent needs is local.</span>
                </li>
                <li className="flex items-start gap-3">
                  <CheckCircle className="mt-1 h-5 w-5 shrink-0 text-brand-500" />
                  <span><strong>Drastically Reduced Build Time:</strong> Experience a smoother, less stressful workflow where AI writes correct FHE code the first time.</span>
                </li>
              </ul>
            </div>
            <div className="relative aspect-square overflow-hidden rounded-2xl border border-white/10 bg-surface-bright/50 shadow-2xl backdrop-blur-sm">
              <div className="flex items-center gap-2 border-b border-white/5 bg-white/5 px-4 py-3">
                <div className="h-3 w-3 rounded-full bg-red-500/80" />
                <div className="h-3 w-3 rounded-full bg-yellow-500/80" />
                <div className="h-3 w-3 rounded-full bg-green-500/80" />
                <span className="ml-2 text-xs font-medium text-white/50">AGENT.md</span>
              </div>
              <div className="p-6 text-sm font-mono text-white/70">
                <div className="mb-4 text-brand-500"># FHEVM AI Developer Guide</div>
                <div className="mb-2 opacity-60">{"// Rules for AI coding assistants"}</div>
                <div className="mb-2">1. Always use <span className="text-emerald-400">TFHE.allowThis()</span> before assigning to state.</div>
                <div className="mb-2">2. Never use FHE operations inside a view function.</div>
                <div className="mb-2">3. Encrypted parameters must be typed as <span className="text-cyan-400">einput</span>.</div>
                <div className="mt-6 opacity-60">{"// Result: 10x Faster Development"}</div>
              </div>
            </div>
          </div>
        </section>

        {/* DevEx Superpowers Section */}
        <section className="py-24 mt-12">
          <div className="mb-16 text-center">
            <h2 className="text-3xl font-bold text-white sm:text-4xl">Developer Experience, Perfected.</h2>
            <p className="mt-4 text-text-muted">Tools that make building confidential smart contracts stress-free.</p>
          </div>

          <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
            <FeatureCard
              title="FHE-Aware Linter"
              description="Catch silent FHEVM bugs instantly. Ensure TFHE.allow() is called and view functions don't perform encrypted operations before you even run tests."
              icon={Code2}
              delay={0.1}
            />
            <FeatureCard
              title="Coprocessor Gas Profiling"
              description="Run fhevm-forge gas to get a detailed breakdown of both on-chain EVM gas and Zama coprocessor gas per TFHE operation."
              icon={Cpu}
              delay={0.2}
            />
            <FeatureCard
              title="Built-in Typescript SDK"
              description="Stop wrestling with WASM initialization. The generated @fhevm-forge/sdk handles relayer logic, EIP-712 reencryption, and client-side encryption hooks out of the box."
              icon={Box}
              delay={0.3}
            />
          </div>
        </section>

      </main>

      {/* Footer */}
      <footer className="border-t border-white/5 bg-surface-bright/20 py-12 text-center text-sm text-text-muted">
        <div className="mx-auto flex max-w-7xl flex-col items-center justify-between gap-6 px-6 sm:flex-row">
          <p>© 2026 fhevm-forge. Open source software.</p>
          <div className="flex gap-6">
            <Link href="https://crates.io/crates/fhevm-forge" className="hover:text-white transition-colors">Crate</Link>
            <Link href="https://www.npmjs.com/package/fhevm-forge-sdk" className="hover:text-white transition-colors">NPM</Link>
            <Link href="https://docs.zama.ai/fhevm" className="hover:text-white transition-colors">Zama Docs</Link>
          </div>
        </div>
      </footer>
    </div>
  );
}

function CheckCircle({ className }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
    >
      <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
      <path d="m9 11 3 3L22 4" />
    </svg>
  );
}
