"use client";

import { useState, useCallback, useEffect } from "react";
import type { TasksResponse, CreateTaskInput, TaskWithDue } from "@/types";

export function useTasks() {
  const [tasks, setTasks] = useState<TasksResponse>({
    overdue: [],
    today: [],
    thisWeek: [],
    upcoming: [],
  });
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchTasks = useCallback(async () => {
    try {
      const response = await fetch("/api/tasks");
      if (!response.ok) {
        throw new Error("Failed to fetch tasks");
      }
      const data: TasksResponse = await response.json();
      setTasks(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch tasks");
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchTasks();
  }, [fetchTasks]);

  const createTask = useCallback(
    async (data: CreateTaskInput) => {
      const response = await fetch("/api/tasks", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to create task");
      }

      await fetchTasks();
    },
    [fetchTasks]
  );

  const updateTask = useCallback(
    async (id: string, data: Partial<CreateTaskInput>) => {
      const response = await fetch(`/api/tasks/${id}`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to update task");
      }

      await fetchTasks();
    },
    [fetchTasks]
  );

  const deleteTask = useCallback(
    async (id: string) => {
      const response = await fetch(`/api/tasks/${id}`, {
        method: "DELETE",
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to delete task");
      }

      await fetchTasks();
    },
    [fetchTasks]
  );

  const completeTask = useCallback(
    async (id: string): Promise<{ daysUntilDue: number }> => {
      const response = await fetch(`/api/tasks/${id}/complete`, {
        method: "POST",
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to complete task");
      }

      const data = await response.json();
      await fetchTasks();
      return { daysUntilDue: data.task.daysUntilDue };
    },
    [fetchTasks]
  );

  return {
    tasks,
    isLoading,
    error,
    fetchTasks,
    createTask,
    updateTask,
    deleteTask,
    completeTask,
  };
}
