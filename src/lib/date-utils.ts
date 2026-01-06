import {
  addDays,
  addWeeks,
  addMonths,
  differenceInDays,
  startOfDay,
} from "date-fns";
import type { IntervalUnit, UrgencyLevel } from "@/types";

export function calculateNextDueDate(
  lastCompletedAt: Date | null,
  intervalValue: number,
  intervalUnit: IntervalUnit,
  createdAt: Date
): Date {
  const baseDate = lastCompletedAt || createdAt;

  switch (intervalUnit) {
    case "days":
      return addDays(baseDate, intervalValue);
    case "weeks":
      return addWeeks(baseDate, intervalValue);
    case "months":
      return addMonths(baseDate, intervalValue);
    default:
      return addDays(baseDate, intervalValue);
  }
}

export function calculateDaysUntilDue(nextDueDate: Date): number {
  const today = startOfDay(new Date());
  const dueDay = startOfDay(nextDueDate);
  return differenceInDays(dueDay, today);
}

export function calculateUrgency(daysUntilDue: number): UrgencyLevel {
  if (daysUntilDue < 0) return "overdue";
  if (daysUntilDue === 0) return "today";
  if (daysUntilDue <= 7) return "this-week";
  return "upcoming";
}

export function formatDaysUntilDue(daysUntilDue: number): string {
  if (daysUntilDue < 0) {
    const overdueDays = Math.abs(daysUntilDue);
    return overdueDays === 1 ? "1 day overdue" : `${overdueDays} days overdue`;
  }
  if (daysUntilDue === 0) return "Due today";
  if (daysUntilDue === 1) return "1 day left";
  return `${daysUntilDue} days left`;
}

export function formatInterval(value: number, unit: IntervalUnit): string {
  if (value === 1) {
    switch (unit) {
      case "days":
        return "Every day";
      case "weeks":
        return "Every week";
      case "months":
        return "Every month";
    }
  }
  return `Every ${value} ${unit}`;
}
