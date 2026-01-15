"use client";

import { cn } from "@/lib/utils";
import { HTMLAttributes, forwardRef } from "react";

interface CardProps extends HTMLAttributes<HTMLDivElement> {
  hover?: boolean;
}

const Card = forwardRef<HTMLDivElement, CardProps>(
  ({ className, hover = false, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          "rounded-2xl p-6 transition-all duration-200",
          "bg-white dark:bg-[var(--neutral-800)]",
          "border border-[var(--border)]",
          hover && [
            "hover:shadow-elevated hover:border-[var(--neutral-300)]",
            "dark:hover:border-[var(--neutral-700)]",
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
