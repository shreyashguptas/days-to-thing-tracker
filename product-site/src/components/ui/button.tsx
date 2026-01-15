"use client";

import { cn } from "@/lib/utils";
import { ButtonHTMLAttributes, forwardRef } from "react";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "outline" | "ghost";
  size?: "sm" | "md" | "lg";
}

const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = "primary", size = "md", children, ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={cn(
          // Base styles - architectural, not pill-shaped
          "relative inline-flex items-center justify-center font-medium transition-all duration-200",
          "rounded-lg focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2",
          "disabled:opacity-50 disabled:cursor-not-allowed",

          // Primary - solid coral with depth
          variant === "primary" && [
            "bg-[var(--accent)] text-white",
            "shadow-subtle hover:bg-[var(--accent-hover)] hover:shadow-elevated",
            "active:scale-[0.98]",
            "focus-visible:ring-[var(--accent)]",
          ],

          // Outline - clean border with proper weight
          variant === "outline" && [
            "border border-[var(--border)] text-[var(--foreground)] bg-transparent",
            "hover:bg-[var(--neutral-100)] dark:hover:bg-[var(--neutral-800)]",
            "focus-visible:ring-[var(--neutral-400)]",
          ],

          // Ghost - minimal
          variant === "ghost" && [
            "text-[var(--muted)] hover:text-[var(--foreground)]",
            "hover:bg-[var(--neutral-100)] dark:hover:bg-[var(--neutral-800)]",
            "focus-visible:ring-[var(--neutral-400)]",
          ],

          // Sizes - refined proportions
          size === "sm" && "px-4 py-2 text-sm",
          size === "md" && "px-5 py-2.5 text-sm",
          size === "lg" && "px-7 py-3.5 text-base tracking-wide",

          className
        )}
        {...props}
      >
        {children}
      </button>
    );
  }
);

Button.displayName = "Button";

export { Button };
