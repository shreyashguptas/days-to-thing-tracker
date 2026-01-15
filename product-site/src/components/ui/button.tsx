"use client";

import { cn } from "@/lib/utils";
import { ButtonHTMLAttributes, forwardRef } from "react";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "outline" | "ghost";
  size?: "sm" | "md" | "lg" | "xl";
}

const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = "primary", size = "md", children, ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={cn(
          "relative inline-flex items-center justify-center font-semibold transition-all duration-300 rounded-full",
          "focus:outline-none focus:ring-2 focus:ring-offset-2",
          "disabled:opacity-50 disabled:cursor-not-allowed",
          // Variants
          variant === "primary" && [
            "bg-gradient-to-r from-[var(--pop-coral)] to-[var(--pop-orange)]",
            "text-white shadow-lg hover:shadow-xl hover:scale-105",
            "focus:ring-[var(--pop-coral)]",
          ],
          variant === "secondary" && [
            "bg-gradient-to-r from-[var(--pop-cyan)] to-[var(--pop-blue)]",
            "text-white shadow-lg hover:shadow-xl hover:scale-105",
            "focus:ring-[var(--pop-blue)]",
          ],
          variant === "outline" && [
            "border-2 border-[var(--pop-coral)] text-[var(--pop-coral)]",
            "hover:bg-[var(--pop-coral)] hover:text-white",
            "focus:ring-[var(--pop-coral)]",
          ],
          variant === "ghost" && [
            "text-[var(--foreground)] hover:bg-black/5 dark:hover:bg-white/10",
            "focus:ring-[var(--pop-blue)]",
          ],
          // Sizes
          size === "sm" && "px-4 py-2 text-sm",
          size === "md" && "px-6 py-3 text-base",
          size === "lg" && "px-8 py-4 text-lg",
          size === "xl" && "px-10 py-5 text-xl",
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
