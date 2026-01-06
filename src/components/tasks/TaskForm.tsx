"use client";

import { useState, useEffect } from "react";
import type { TaskWithDue, IntervalUnit, CreateTaskInput } from "@/types";
import { Dialog, DialogClose } from "@/components/ui/Dialog";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";

interface TaskFormProps {
  open: boolean;
  onClose: () => void;
  onSubmit: (data: CreateTaskInput) => Promise<void>;
  task?: TaskWithDue | null;
}

export function TaskForm({ open, onClose, onSubmit, task }: TaskFormProps) {
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [intervalValueStr, setIntervalValueStr] = useState("1");
  const [intervalUnit, setIntervalUnit] = useState<IntervalUnit>("months");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    if (task) {
      setName(task.name);
      setDescription(task.description || "");
      setIntervalValueStr(String(task.intervalValue));
      setIntervalUnit(task.intervalUnit);
    } else {
      setName("");
      setDescription("");
      setIntervalValueStr("1");
      setIntervalUnit("months");
    }
    setError("");
  }, [task, open]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    if (!name.trim()) {
      setError("Name is required");
      return;
    }

    const intervalValue = parseInt(intervalValueStr) || 0;
    if (intervalValue < 1) {
      setError("Interval must be at least 1");
      return;
    }

    setIsSubmitting(true);
    try {
      await onSubmit({
        name: name.trim(),
        description: description.trim() || undefined,
        intervalValue,
        intervalUnit,
      });
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save task");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onClose={onClose} title={task ? "Edit Task" : "Add Task"}>
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

      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label
            htmlFor="name"
            className="block text-sm font-medium mb-1"
          >
            Name
          </label>
          <Input
            id="name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g., Change water filter"
            autoFocus
          />
        </div>

        <div>
          <label
            htmlFor="description"
            className="block text-sm font-medium mb-1"
          >
            Description (optional)
          </label>
          <Input
            id="description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="e.g., Kitchen sink filter"
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">
            Repeat every
          </label>
          <div className="grid grid-cols-2 gap-2">
            <Input
              type="number"
              min={1}
              value={intervalValueStr}
              onChange={(e) => setIntervalValueStr(e.target.value)}
            />
            <Select
              value={intervalUnit}
              onChange={(e) => setIntervalUnit(e.target.value as IntervalUnit)}
            >
              <option value="days">Days</option>
              <option value="weeks">Weeks</option>
              <option value="months">Months</option>
            </Select>
          </div>
        </div>

        {error && (
          <p className="text-sm text-red-500">{error}</p>
        )}

        <div className="flex gap-2 justify-end pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" disabled={isSubmitting}>
            {isSubmitting ? "Saving..." : task ? "Update" : "Add Task"}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
