"use client";

import { cn } from "@/lib/utils";
import { HTMLAttributes } from "react";

interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  variant?: "coral" | "orange" | "mint" | "cyan" | "blue" | "purple" | "pink" | "yellow";
}

export function Badge({ className, variant = "coral", children, ...props }: BadgeProps) {
  const variantStyles = {
    coral: "bg-[var(--pop-coral)]/10 text-[var(--pop-coral)] border-[var(--pop-coral)]/30",
    orange: "bg-[var(--pop-orange)]/10 text-[var(--pop-orange)] border-[var(--pop-orange)]/30",
    mint: "bg-[var(--pop-mint)]/10 text-[var(--pop-mint)] border-[var(--pop-mint)]/30",
    cyan: "bg-[var(--pop-cyan)]/10 text-[var(--pop-cyan)] border-[var(--pop-cyan)]/30",
    blue: "bg-[var(--pop-blue)]/10 text-[var(--pop-blue)] border-[var(--pop-blue)]/30",
    purple: "bg-[var(--pop-purple)]/10 text-[var(--pop-purple)] border-[var(--pop-purple)]/30",
    pink: "bg-[var(--pop-pink)]/10 text-[var(--pop-pink)] border-[var(--pop-pink)]/30",
    yellow: "bg-[var(--pop-yellow)]/10 text-[var(--pop-yellow)] border-[var(--pop-yellow)]/30",
  };

  return (
    <span
      className={cn(
        "inline-flex items-center px-3 py-1 rounded-full text-sm font-medium border",
        variantStyles[variant],
        className
      )}
      {...props}
    >
      {children}
    </span>
  );
}
