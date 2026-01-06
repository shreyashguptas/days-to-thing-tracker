"use client";

import type { ReactNode } from "react";
import type { UrgencyLevel } from "@/types";

interface TaskSectionProps {
  title: string;
  urgency: UrgencyLevel;
  count: number;
  children: ReactNode;
}

export function TaskSection({
  title,
  urgency,
  count,
  children,
}: TaskSectionProps) {
  if (count === 0) return null;

  const dotColors: Record<UrgencyLevel, string> = {
    overdue: "bg-[var(--urgency-overdue)]",
    today: "bg-[var(--urgency-today)]",
    "this-week": "bg-[var(--urgency-week)]",
    upcoming: "bg-[var(--urgency-upcoming)]",
  };

  return (
    <section className="mb-8">
      <div className="flex items-center gap-2 mb-4">
        <div
          className={`w-2 h-2 rounded-full ${dotColors[urgency]} ${
            urgency === "overdue" ? "animate-pulse-urgency" : ""
          }`}
        />
        <h2 className="text-lg font-semibold">{title}</h2>
        <span className="text-sm text-muted-foreground">({count})</span>
      </div>
      <div className="space-y-3">{children}</div>
    </section>
  );
}
