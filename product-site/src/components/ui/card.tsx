"use client";

import { cn } from "@/lib/utils";
import { HTMLAttributes, forwardRef } from "react";

interface CardProps extends HTMLAttributes<HTMLDivElement> {
  variant?: "default" | "glass" | "gradient";
  hover?: boolean;
}

const Card = forwardRef<HTMLDivElement, CardProps>(
  ({ className, variant = "default", hover = false, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          "rounded-3xl p-6 transition-all duration-300",
          variant === "default" && [
            "bg-white dark:bg-[var(--dark-card)]",
            "border border-gray-100 dark:border-[var(--dark-border)]",
            "shadow-sm",
          ],
          variant === "glass" && [
            "bg-white/10 dark:bg-white/5",
            "backdrop-blur-xl",
            "border border-white/20",
          ],
          variant === "gradient" && [
            "bg-gradient-to-br from-white to-gray-50",
            "dark:from-[var(--dark-card)] dark:to-[var(--dark-bg)]",
            "border border-gray-100 dark:border-[var(--dark-border)]",
          ],
          hover && [
            "hover:shadow-xl hover:scale-[1.02]",
            "cursor-pointer",
          ],
          className
        )}
        {...props}
      >
        {children}
      </div>
    );
  }
);

Card.displayName = "Card";

export { Card };
