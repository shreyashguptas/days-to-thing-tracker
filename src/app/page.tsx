"use client";

import { useState, Suspense } from "react";
import { useSearchParams } from "next/navigation";
import { ThemeToggle } from "@/components/ui/ThemeToggle";
import { Button } from "@/components/ui/Button";
import { TaskList } from "@/components/tasks/TaskList";
import { TaskForm } from "@/components/tasks/TaskForm";
import { KioskView } from "@/components/kiosk";
import { useTasks } from "@/hooks/useTasks";
import { useCountdown } from "@/hooks/useCountdown";
import type { TaskWithDue, CreateTaskInput } from "@/types";

// Inner component that uses useSearchParams
function HomeContent() {
  const searchParams = useSearchParams();
  const isKioskMode = searchParams.get("kiosk") === "true";

  const {
    tasks,
    isLoading,
    error,
    createTask,
    updateTask,
    deleteTask,
    completeTask,
    fetchTasks,
  } = useTasks();
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [editingTask, setEditingTask] = useState<TaskWithDue | null>(null);

  // Refresh data every minute (30 seconds in kiosk mode for quicker updates)
  useCountdown(isKioskMode ? 30000 : 60000);

  const handleOpenForm = () => {
    setEditingTask(null);
    setIsFormOpen(true);
  };

  const handleEdit = (task: TaskWithDue) => {
    setEditingTask(task);
    setIsFormOpen(true);
  };

  const handleCloseForm = () => {
    setIsFormOpen(false);
    setEditingTask(null);
  };

  const handleSubmit = async (data: CreateTaskInput) => {
    if (editingTask) {
      await updateTask(editingTask.id, data);
    } else {
      await createTask(data);
    }
  };

  const totalTasks =
    tasks.overdue.length +
    tasks.today.length +
    tasks.thisWeek.length +
    tasks.upcoming.length;

  // Kiosk mode: simplified UI for 160x128 TFT display with rotary encoder
  if (isKioskMode) {
    if (isLoading) {
      return (
        <div className="kiosk-container">
          <div className="kiosk-loading">
            <div className="kiosk-loading-spinner" />
            <div className="kiosk-loading-text">Loading...</div>
          </div>
        </div>
      );
    }

    if (error) {
      return (
        <div className="kiosk-container">
          <div className="kiosk-feedback">
            <div className="kiosk-feedback-icon" style={{ color: 'var(--urgency-overdue)' }}>!</div>
            <div className="kiosk-feedback-text">Error</div>
          </div>
        </div>
      );
    }

    return (
      <KioskView
        tasks={tasks}
        onComplete={async (id) => { await completeTask(id); }}
        onDelete={deleteTask}
      />
    );
  }

  // Normal mode: full UI
  return (
    <div className="min-h-screen bg-background">
      <header className="sticky top-0 z-40 border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="container mx-auto px-3 sm:px-4 h-14 sm:h-16 flex items-center justify-between gap-2">
          <div className="min-w-0">
            <h1 className="text-lg sm:text-xl font-semibold truncate">Days to Thing</h1>
            {!isLoading && totalTasks > 0 && (
              <p className="text-xs text-muted-foreground">
                {totalTasks} task{totalTasks !== 1 ? "s" : ""}
              </p>
            )}
          </div>
          <div className="flex items-center gap-1.5 sm:gap-2 shrink-0">
            <Button onClick={handleOpenForm} size="sm" className="sm:h-10 sm:px-4 sm:text-sm">
              <svg
                className="w-4 h-4 sm:mr-1"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={2}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M12 4v16m8-8H4"
                />
              </svg>
              <span className="hidden sm:inline">Add Task</span>
            </Button>
            <ThemeToggle />
          </div>
        </div>
      </header>

      <main className="container mx-auto px-3 sm:px-4 py-4 sm:py-8 max-w-2xl">
        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-foreground" />
          </div>
        ) : error ? (
          <div className="text-center py-12">
            <p className="text-red-500 mb-4">{error}</p>
            <Button onClick={fetchTasks}>Try Again</Button>
          </div>
        ) : (
          <TaskList
            tasks={tasks}
            onComplete={completeTask}
            onEdit={handleEdit}
            onDelete={deleteTask}
          />
        )}
      </main>

      <TaskForm
        open={isFormOpen}
        onClose={handleCloseForm}
        onSubmit={handleSubmit}
        task={editingTask}
      />
    </div>
  );
}

// Loading fallback for Suspense
function LoadingFallback() {
  return (
    <div className="min-h-screen bg-background flex items-center justify-center">
      <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-foreground" />
    </div>
  );
}

// Main export wrapped in Suspense for useSearchParams
export default function Home() {
  return (
    <Suspense fallback={<LoadingFallback />}>
      <HomeContent />
    </Suspense>
  );
}
