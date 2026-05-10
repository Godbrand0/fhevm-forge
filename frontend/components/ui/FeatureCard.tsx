"use client";

import { motion } from "framer-motion";
import { type LucideIcon } from "lucide-react";
import { cn } from "@/lib/utils";
import { ReactNode } from "react";

interface FeatureCardProps {
  title: string;
  description: string | ReactNode;
  icon: LucideIcon;
  delay?: number;
  className?: string;
}

export function FeatureCard({ title, description, icon: Icon, delay = 0, className }: FeatureCardProps) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ duration: 0.5, delay }}
      className={cn(
        "group relative overflow-hidden rounded-2xl border border-white/10 bg-surface-dim/50 p-8 transition-colors hover:bg-surface-bright/50",
        className
      )}
    >
      <div className="absolute inset-0 bg-gradient-to-b from-brand-500/5 to-transparent opacity-0 transition-opacity group-hover:opacity-100" />
      <div className="relative z-10 flex flex-col gap-4">
        <div className="inline-flex h-12 w-12 items-center justify-center rounded-lg bg-brand-500/10 text-brand-500 ring-1 ring-brand-500/20">
          <Icon className="h-6 w-6" />
        </div>
        <h3 className="text-xl font-semibold text-white">{title}</h3>
        <div className="text-text-muted leading-relaxed">{description}</div>
      </div>
    </motion.div>
  );
}
