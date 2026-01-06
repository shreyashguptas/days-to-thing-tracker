"use client";

import { useState } from "react";
import type { TaskWithDue } from "@/types";
import { formatInterval, formatLastCompleted } from "@/lib/date-utils";
import { Card } from "@/components/ui/Card";
import { Button } from "@/components/ui/Button";
import { useToast } from "@/components/ui/Toast";
import { UrgencyIndicator } from "./UrgencyIndicator";
import { CountdownDisplay } from "./CountdownDisplay";
import { HistoryModal } from "./HistoryModal";

interface TaskCardProps {
  task: TaskWithDue;
  onComplete: (id: string) => Promise<{ daysUntilDue: number }>;
  onEdit: (task: TaskWithDue) => void;
  onDelete: (id: string) => Promise<void>;
}

export function TaskCard({ task, onComplete, onEdit, onDelete }: TaskCardProps) {
  const [isCompleting, setIsCompleting] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [showSuccess, setShowSuccess] = useState(false);
  const [showHistory, setShowHistory] = useState(false);
  const { showToast } = useToast();

  const handleComplete = async () => {
    setIsCompleting(true);
    try {
      const result = await onComplete(task.id);
      setShowSuccess(true);
      showToast(`Task completed! Next due in ${result.daysUntilDue} days`);
      setTimeout(() => setShowSuccess(false), 600);
    } catch {
      showToast("Failed to complete task", "error");
    } finally {
      setIsCompleting(false);
    }
  };

  const handleDelete = async () => {
    setIsDeleting(true);
    try {
      await onDelete(task.id);
    } finally {
      setIsDeleting(false);
      setShowConfirm(false);
    }
  };

  return (
    <Card className={`relative overflow-hidden ${showSuccess ? "animate-success-pulse" : ""}`}>
      <div className="flex">
        <UrgencyIndicator urgency={task.urgency} />

        <div className="flex-1 p-3 sm:p-4 min-w-0">
          {/* Header row with name and countdown */}
          <div className="flex items-start justify-between gap-2 sm:gap-4">
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-1.5">
                <h3 className="font-medium text-sm sm:text-base truncate">{task.name}</h3>
                <button
                  onClick={() => setShowHistory(true)}
                  className="shrink-0 p-1 rounded hover:bg-card text-muted hover:text-foreground transition-colors"
                  title="View completion history"
                  aria-label="View history"
                >
                  <svg className="w-3.5 h-3.5 sm:w-4 sm:h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                </button>
              </div>
              {task.description && (
                <p className="text-xs sm:text-sm text-muted-foreground mt-0.5 sm:mt-1 line-clamp-2">
                  {task.description}
                </p>
              )}
            </div>

            <CountdownDisplay
              daysUntilDue={task.daysUntilDue}
              urgency={task.urgency}
            />
          </div>

          {/* Meta info */}
          <div className="flex flex-wrap items-center gap-x-2 gap-y-0.5 mt-1.5 sm:mt-2 text-[10px] sm:text-xs text-muted">
            <span>{formatInterval(task.intervalValue, task.intervalUnit)}</span>
            <span>•</span>
            <span>Last: {formatLastCompleted(task.lastCompletedAt)}</span>
          </div>

          {/* Actions */}
          <div className="flex flex-wrap items-center gap-1.5 sm:gap-2 mt-3 sm:mt-4">
            <Button
              size="sm"
              onClick={handleComplete}
              disabled={isCompleting}
              className="text-xs sm:text-sm h-7 sm:h-8 px-2 sm:px-3"
            >
              {isCompleting ? "..." : "Done"}
            </Button>

            <Button
              size="sm"
              variant="ghost"
              onClick={() => onEdit(task)}
              className="text-xs sm:text-sm h-7 sm:h-8 px-2 sm:px-3"
            >
              Edit
            </Button>

            {showConfirm ? (
              <>
                <Button
                  size="sm"
                  variant="danger"
                  onClick={handleDelete}
                  disabled={isDeleting}
                  className="text-xs sm:text-sm h-7 sm:h-8 px-2 sm:px-3"
                >
                  {isDeleting ? "..." : "Yes"}
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => setShowConfirm(false)}
                  className="text-xs sm:text-sm h-7 sm:h-8 px-2 sm:px-3"
                >
                  No
                </Button>
              </>
            ) : (
              <Button
                size="sm"
                variant="ghost"
                onClick={() => setShowConfirm(true)}
                className="text-xs sm:text-sm h-7 sm:h-8 px-2 sm:px-3"
              >
                Delete
              </Button>
            )}
          </div>
        </div>
      </div>

      <HistoryModal
        open={showHistory}
        onClose={() => setShowHistory(false)}
        taskId={task.id}
        taskName={task.name}
      />
    </Card>
  );
}
