"use client";

import { useState, useEffect } from "react";
import { Dialog, DialogClose } from "@/components/ui/Dialog";
import { formatLastCompleted } from "@/lib/date-utils";

interface Completion {
  id: string;
  completedAt: string;
}

interface HistoryModalProps {
  open: boolean;
  onClose: () => void;
  taskId: string;
  taskName: string;
}

export function HistoryModal({ open, onClose, taskId, taskName }: HistoryModalProps) {
  const [completions, setCompletions] = useState<Completion[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open && taskId) {
      setIsLoading(true);
      setError(null);

      fetch(`/api/tasks/${taskId}/history`)
        .then((res) => res.json())
        .then((data) => {
          if (data.error) {
            setError(data.error);
          } else {
            setCompletions(data.completions || []);
          }
        })
        .catch(() => setError("Failed to load history"))
        .finally(() => setIsLoading(false));
    }
  }, [open, taskId]);

  return (
    <Dialog open={open} onClose={onClose} title={`History: ${taskName}`}>
      <DialogClose onClose={onClose}>
        <svg
          className="w-5 h-5"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          strokeWidth={2}
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M6 18L18 6M6 6l12 12"
          />
        </svg>
      </DialogClose>

      <div className="mt-2">
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-foreground" />
          </div>
        ) : error ? (
          <p className="text-red-500 text-sm py-4">{error}</p>
        ) : completions.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            <svg
              className="w-12 h-12 mx-auto mb-3 opacity-50"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              strokeWidth={1.5}
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <p className="text-sm">No completions yet</p>
            <p className="text-xs mt-1">Mark this task as done to start tracking</p>
          </div>
        ) : (
          <ul className="space-y-2 max-h-64 overflow-y-auto">
            {completions.map((completion, index) => (
              <li
                key={completion.id}
                className="flex items-center gap-3 py-2 px-3 rounded-lg bg-card"
              >
                <div className="w-6 h-6 rounded-full bg-green-500/20 flex items-center justify-center shrink-0">
                  <svg
                    className="w-3.5 h-3.5 text-green-500"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    strokeWidth={2.5}
                  >
                    <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                  </svg>
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium">
                    {formatLastCompleted(completion.completedAt)}
                  </p>
                  <p className="text-xs text-muted-foreground">
                    {new Date(completion.completedAt).toLocaleDateString("en-US", {
                      weekday: "long",
                      year: "numeric",
                      month: "long",
                      day: "numeric",
                    })}
                  </p>
                </div>
                {index === 0 && (
                  <span className="text-xs bg-green-500/20 text-green-600 dark:text-green-400 px-2 py-0.5 rounded">
                    Latest
                  </span>
                )}
              </li>
            ))}
          </ul>
        )}

        {completions.length > 0 && (
          <p className="text-xs text-muted-foreground mt-4 text-center">
            Showing last {completions.length} completion{completions.length !== 1 ? "s" : ""}
          </p>
        )}
      </div>
    </Dialog>
  );
}
