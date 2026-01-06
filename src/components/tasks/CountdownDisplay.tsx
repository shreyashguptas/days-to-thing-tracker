"use client";

import type { UrgencyLevel } from "@/types";

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

  // Format the display text
  const getDisplayText = () => {
    if (daysUntilDue < 0) {
      return { number: displayNumber, suffix: displayNumber === 1 ? "day overdue" : "days overdue" };
    }
    if (daysUntilDue === 0) {
      return { number: null, suffix: "Due today" };
    }
    return { number: displayNumber, suffix: displayNumber === 1 ? "day left" : "days left" };
  };

  const { number, suffix } = getDisplayText();

  return (
    <div className="flex items-baseline gap-1.5 shrink-0">
      {number !== null && (
        <span
          className={`text-xl sm:text-2xl font-bold tabular-nums ${textColors[urgency]}`}
          key={displayNumber}
        >
          {number}
        </span>
      )}
      <span className={`text-xs sm:text-sm ${number === null ? textColors[urgency] + " font-semibold" : "text-muted-foreground"}`}>
        {suffix}
      </span>
    </div>
  );
}
