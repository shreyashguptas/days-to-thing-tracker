"use client";

import type { UrgencyLevel } from "@/types";
import { formatDaysUntilDue } from "@/lib/date-utils";

interface CountdownDisplayProps {
  daysUntilDue: number;
  urgency: UrgencyLevel;
}

export function CountdownDisplay({
  daysUntilDue,
  urgency,
}: CountdownDisplayProps) {
  const textColors: Record<UrgencyLevel, string> = {
    overdue: "text-[var(--urgency-overdue)]",
    today: "text-[var(--urgency-today)]",
    "this-week": "text-[var(--urgency-week)]",
    upcoming: "text-[var(--urgency-upcoming)]",
  };

  const displayNumber = Math.abs(daysUntilDue);
  const text = formatDaysUntilDue(daysUntilDue);

  return (
    <div className="flex flex-col items-end">
      <span
        className={`text-2xl font-bold tabular-nums ${textColors[urgency]}`}
        key={displayNumber}
      >
        {displayNumber}
      </span>
      <span className="text-xs text-muted-foreground">{text}</span>
    </div>
  );
}
