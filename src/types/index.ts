export type IntervalUnit = "days" | "weeks" | "months";

export type UrgencyLevel = "overdue" | "today" | "this-week" | "upcoming";

export interface Task {
  id: string;
  name: string;
  description: string | null;
  intervalValue: number;
  intervalUnit: IntervalUnit;
  lastCompletedAt: string | null;
  createdAt: string;
  updatedAt: string;
  isArchived: boolean;
}

export interface TaskWithDue extends Task {
  nextDueDate: string;
  daysUntilDue: number;
  urgency: UrgencyLevel;
}

export interface TasksResponse {
  overdue: TaskWithDue[];
  today: TaskWithDue[];
  thisWeek: TaskWithDue[];
  upcoming: TaskWithDue[];
}

export interface CreateTaskInput {
  name: string;
  description?: string;
  intervalValue: number;
  intervalUnit: IntervalUnit;
}

export interface UpdateTaskInput extends Partial<CreateTaskInput> {}
