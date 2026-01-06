"use client";

import type { TasksResponse, TaskWithDue, CreateTaskInput } from "@/types";
import { TaskCard } from "./TaskCard";
import { TaskSection } from "./TaskSection";

interface TaskListProps {
  tasks: TasksResponse;
  onComplete: (id: string) => Promise<{ daysUntilDue: number }>;
  onEdit: (task: TaskWithDue) => void;
  onDelete: (id: string) => Promise<void>;
}

export function TaskList({ tasks, onComplete, onEdit, onDelete }: TaskListProps) {
  const hasAnyTasks =
    tasks.overdue.length > 0 ||
    tasks.today.length > 0 ||
    tasks.thisWeek.length > 0 ||
    tasks.upcoming.length > 0;

  if (!hasAnyTasks) {
    return (
      <div className="text-center py-12">
        <div className="text-4xl mb-4">
          <svg
            className="w-16 h-16 mx-auto text-muted"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={1.5}
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        </div>
        <h3 className="text-lg font-medium mb-2">No tasks yet</h3>
        <p className="text-muted-foreground">
          Add your first recurring task to get started
        </p>
      </div>
    );
  }

  return (
    <div>
      <TaskSection
        title="Overdue"
        urgency="overdue"
        count={tasks.overdue.length}
      >
        {tasks.overdue.map((task) => (
          <TaskCard
            key={task.id}
            task={task}
            onComplete={onComplete}
            onEdit={onEdit}
            onDelete={onDelete}
          />
        ))}
      </TaskSection>

      <TaskSection title="Due Today" urgency="today" count={tasks.today.length}>
        {tasks.today.map((task) => (
          <TaskCard
            key={task.id}
            task={task}
            onComplete={onComplete}
            onEdit={onEdit}
            onDelete={onDelete}
          />
        ))}
      </TaskSection>

      <TaskSection
        title="This Week"
        urgency="this-week"
        count={tasks.thisWeek.length}
      >
        {tasks.thisWeek.map((task) => (
          <TaskCard
            key={task.id}
            task={task}
            onComplete={onComplete}
            onEdit={onEdit}
            onDelete={onDelete}
          />
        ))}
      </TaskSection>

      <TaskSection
        title="Coming Up"
        urgency="upcoming"
        count={tasks.upcoming.length}
      >
        {tasks.upcoming.map((task) => (
          <TaskCard
            key={task.id}
            task={task}
            onComplete={onComplete}
            onEdit={onEdit}
            onDelete={onDelete}
          />
        ))}
      </TaskSection>
    </div>
  );
}
