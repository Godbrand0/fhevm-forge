"use client";

import { Check, Copy } from "lucide-react";
import { useState } from "react";
import { cn } from "@/lib/utils";

interface CodeBlockProps {
  code: string;
  language?: string;
  className?: string;
}

export function CodeBlock({ code, language = "bash", className }: CodeBlockProps) {
  const [copied, setCopied] = useState(false);

  const onCopy = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div
      className={cn(
        "relative flex items-center justify-between rounded-xl bg-surface-dim border border-white/10 px-4 py-3 font-mono text-sm shadow-xl",
        className
      )}
    >
      <div className="flex items-center gap-3 overflow-x-auto text-brand-500">
        <span className="text-white/30 select-none">$</span>
        <code className="text-white/90">{code}</code>
      </div>
      <button
        onClick={onCopy}
        className="ml-4 flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-white/5 bg-white/5 text-white/50 transition-all hover:bg-white/10 hover:text-white"
        aria-label="Copy code"
      >
        {copied ? <Check className="h-4 w-4 text-emerald-400" /> : <Copy className="h-4 w-4" />}
      </button>
    </div>
  );
}
