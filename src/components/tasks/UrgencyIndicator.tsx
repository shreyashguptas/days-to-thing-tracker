"use client";

import type { UrgencyLevel } from "@/types";

interface UrgencyIndicatorProps {
  urgency: UrgencyLevel;
}

export function UrgencyIndicator({ urgency }: UrgencyIndicatorProps) {
  const styles: Record<UrgencyLevel, string> = {
    overdue: "bg-[var(--urgency-overdue)] animate-pulse-urgency",
    today: "bg-[var(--urgency-today)]",
    "this-week": "bg-[var(--urgency-week)]",
    upcoming: "bg-[var(--urgency-upcoming)]",
  };

  return (
    <div
      className={`w-1 h-full rounded-full ${styles[urgency]}`}
      aria-hidden="true"
    />
  );
}
