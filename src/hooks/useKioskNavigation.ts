'use client';

import { useState, useCallback, useEffect } from 'react';
import { TaskWithDue } from '@/types';

// Kiosk navigation states
export type KioskState =
  | 'TASK_LIST'      // Viewing tasks, rotate to change
  | 'TASK_ACTIONS'   // Viewing actions for selected task
  | 'DELETE_CONFIRM' // Confirming delete
  | 'COMPLETING';    // Showing completion feedback

// Actions available for a task
export type TaskAction = 'done' | 'delete' | 'back';
const TASK_ACTIONS: TaskAction[] = ['done', 'delete', 'back'];

// Delete confirmation options
export type ConfirmOption = 'yes' | 'no';
const CONFIRM_OPTIONS: ConfirmOption[] = ['yes', 'no'];

interface UseKioskNavigationProps {
  tasks: TaskWithDue[];
  onComplete: (taskId: string) => Promise<void>;
  onDelete: (taskId: string) => Promise<void>;
}

interface UseKioskNavigationReturn {
  // Current state
  state: KioskState;

  // Selected indices
  taskIndex: number;
  actionIndex: number;
  confirmIndex: number;

  // Current selections
  currentTask: TaskWithDue | null;
  currentAction: TaskAction;
  currentConfirm: ConfirmOption;

  // Navigation functions
  moveUp: () => void;
  moveDown: () => void;
  select: () => void;
  back: () => void;

  // State info
  isLoading: boolean;
  feedbackMessage: string | null;
}

export function useKioskNavigation({
  tasks,
  onComplete,
  onDelete,
}: UseKioskNavigationProps): UseKioskNavigationReturn {
  // State machine
  const [state, setState] = useState<KioskState>('TASK_LIST');

  // Selection indices
  const [taskIndex, setTaskIndex] = useState(0);
  const [actionIndex, setActionIndex] = useState(0);
  const [confirmIndex, setConfirmIndex] = useState(1); // Default to "No"

  // Loading and feedback
  const [isLoading, setIsLoading] = useState(false);
  const [feedbackMessage, setFeedbackMessage] = useState<string | null>(null);

  // Ensure taskIndex is valid when tasks change
  useEffect(() => {
    if (tasks.length === 0) {
      setTaskIndex(0);
    } else if (taskIndex >= tasks.length) {
      setTaskIndex(tasks.length - 1);
    }
  }, [tasks.length, taskIndex]);

  // Clear feedback after delay
  useEffect(() => {
    if (feedbackMessage) {
      const timer = setTimeout(() => {
        setFeedbackMessage(null);
        setState('TASK_LIST');
      }, 1500);
      return () => clearTimeout(timer);
    }
  }, [feedbackMessage]);

  // Current selections
  const currentTask = tasks.length > 0 ? tasks[taskIndex] : null;
  const currentAction = TASK_ACTIONS[actionIndex];
  const currentConfirm = CONFIRM_OPTIONS[confirmIndex];

  // Move focus up (counter-clockwise rotation)
  const moveUp = useCallback(() => {
    switch (state) {
      case 'TASK_LIST':
        setTaskIndex((prev) => (prev > 0 ? prev - 1 : tasks.length - 1));
        break;
      case 'TASK_ACTIONS':
        setActionIndex((prev) => (prev > 0 ? prev - 1 : TASK_ACTIONS.length - 1));
        break;
      case 'DELETE_CONFIRM':
        setConfirmIndex((prev) => (prev > 0 ? prev - 1 : CONFIRM_OPTIONS.length - 1));
        break;
    }
  }, [state, tasks.length]);

  // Move focus down (clockwise rotation)
  const moveDown = useCallback(() => {
    switch (state) {
      case 'TASK_LIST':
        setTaskIndex((prev) => (prev < tasks.length - 1 ? prev + 1 : 0));
        break;
      case 'TASK_ACTIONS':
        setActionIndex((prev) => (prev < TASK_ACTIONS.length - 1 ? prev + 1 : 0));
        break;
      case 'DELETE_CONFIRM':
        setConfirmIndex((prev) => (prev < CONFIRM_OPTIONS.length - 1 ? prev + 1 : 0));
        break;
    }
  }, [state, tasks.length]);

  // Select current item (encoder press)
  const select = useCallback(async () => {
    if (isLoading) return;

    switch (state) {
      case 'TASK_LIST':
        if (currentTask) {
          setActionIndex(0);
          setState('TASK_ACTIONS');
        }
        break;

      case 'TASK_ACTIONS':
        if (!currentTask) return;

        switch (currentAction) {
          case 'done':
            setIsLoading(true);
            setState('COMPLETING');
            try {
              await onComplete(currentTask.id);
              setFeedbackMessage('Done!');
            } catch {
              setFeedbackMessage('Error');
              setState('TASK_LIST');
            } finally {
              setIsLoading(false);
            }
            break;

          case 'delete':
            setConfirmIndex(1); // Default to "No"
            setState('DELETE_CONFIRM');
            break;

          case 'back':
            setState('TASK_LIST');
            break;
        }
        break;

      case 'DELETE_CONFIRM':
        if (!currentTask) return;

        if (currentConfirm === 'yes') {
          setIsLoading(true);
          try {
            await onDelete(currentTask.id);
            setFeedbackMessage('Deleted');
            // Adjust index if we deleted the last item
            if (taskIndex >= tasks.length - 1 && taskIndex > 0) {
              setTaskIndex(taskIndex - 1);
            }
          } catch {
            setFeedbackMessage('Error');
          } finally {
            setIsLoading(false);
          }
        } else {
          setState('TASK_ACTIONS');
        }
        break;
    }
  }, [state, currentTask, currentAction, currentConfirm, isLoading, onComplete, onDelete, taskIndex, tasks.length]);

  // Go back (long press / escape)
  const back = useCallback(() => {
    switch (state) {
      case 'TASK_ACTIONS':
        setState('TASK_LIST');
        break;
      case 'DELETE_CONFIRM':
        setState('TASK_ACTIONS');
        break;
      case 'COMPLETING':
        // Can't interrupt completion
        break;
      default:
        // Already at top level
        break;
    }
  }, [state]);

  // Keyboard event handler
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case 'ArrowUp':
          e.preventDefault();
          moveUp();
          break;
        case 'ArrowDown':
          e.preventDefault();
          moveDown();
          break;
        case 'Enter':
          e.preventDefault();
          select();
          break;
        case 'Escape':
          e.preventDefault();
          back();
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [moveUp, moveDown, select, back]);

  return {
    state,
    taskIndex,
    actionIndex,
    confirmIndex,
    currentTask,
    currentAction,
    currentConfirm,
    moveUp,
    moveDown,
    select,
    back,
    isLoading,
    feedbackMessage,
  };
}
