"use client";

import { cn } from "@/lib/utils";
import { HTMLAttributes } from "react";

interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  variant?: "default" | "muted";
}

export function Badge({ className, variant = "default", children, ...props }: BadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center px-3 py-1 rounded-full text-xs font-medium tracking-wide uppercase",
        variant === "default" && [
          "bg-[var(--accent-light)] text-[var(--accent)]",
        ],
        variant === "muted" && [
          "bg-[var(--neutral-100)] text-[var(--muted)]",
          "dark:bg-[var(--neutral-800)]",
        ],
        className
      )}
      {...props}
    >
      {children}
    </span>
  );
}
