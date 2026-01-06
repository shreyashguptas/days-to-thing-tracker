"use client";

import { useState } from "react";
import type { TaskWithDue } from "@/types";
import { formatInterval } from "@/lib/date-utils";
import { Card } from "@/components/ui/Card";
import { Button } from "@/components/ui/Button";
import { UrgencyIndicator } from "./UrgencyIndicator";
import { CountdownDisplay } from "./CountdownDisplay";

interface TaskCardProps {
  task: TaskWithDue;
  onComplete: (id: string) => Promise<void>;
  onEdit: (task: TaskWithDue) => void;
  onDelete: (id: string) => Promise<void>;
}

export function TaskCard({ task, onComplete, onEdit, onDelete }: TaskCardProps) {
  const [isCompleting, setIsCompleting] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);

  const handleComplete = async () => {
    setIsCompleting(true);
    try {
      await onComplete(task.id);
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
    <Card className="relative overflow-hidden">
      <div className="flex">
        <UrgencyIndicator urgency={task.urgency} />

        <div className="flex-1 p-4">
          <div className="flex items-start justify-between gap-4">
            <div className="flex-1 min-w-0">
              <h3 className="font-medium truncate">{task.name}</h3>
              {task.description && (
                <p className="text-sm text-muted-foreground mt-1 line-clamp-2">
                  {task.description}
                </p>
              )}
              <p className="text-xs text-muted mt-2">
                {formatInterval(task.intervalValue, task.intervalUnit)}
              </p>
            </div>

            <CountdownDisplay
              daysUntilDue={task.daysUntilDue}
              urgency={task.urgency}
            />
          </div>

          <div className="flex items-center gap-2 mt-4">
            <Button
              size="sm"
              onClick={handleComplete}
              disabled={isCompleting}
            >
              {isCompleting ? "Marking..." : "Mark Done"}
            </Button>

            <Button
              size="sm"
              variant="ghost"
              onClick={() => onEdit(task)}
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
                >
                  {isDeleting ? "..." : "Confirm"}
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => setShowConfirm(false)}
                >
                  Cancel
                </Button>
              </>
            ) : (
              <Button
                size="sm"
                variant="ghost"
                onClick={() => setShowConfirm(true)}
              >
                Delete
              </Button>
            )}
          </div>
        </div>
      </div>
    </Card>
  );
}
