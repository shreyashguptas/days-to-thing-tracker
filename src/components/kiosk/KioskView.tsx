'use client';

import { useMemo } from 'react';
import { TasksResponse, TaskWithDue } from '@/types';
import { useKioskNavigation } from '@/hooks/useKioskNavigation';
import { KioskTaskCard } from './KioskTaskCard';
import { KioskActionMenu } from './KioskActionMenu';
import { KioskConfirmDialog } from './KioskConfirmDialog';

interface KioskViewProps {
  tasks: TasksResponse;
  onComplete: (taskId: string) => Promise<void>;
  onDelete: (taskId: string) => Promise<void>;
}

export function KioskView({ tasks, onComplete, onDelete }: KioskViewProps) {
  // Flatten all tasks into a single list, ordered by urgency
  const allTasks: TaskWithDue[] = useMemo(() => {
    return [
      ...tasks.overdue,
      ...tasks.today,
      ...tasks.thisWeek,
      ...tasks.upcoming,
    ];
  }, [tasks]);

  // Kiosk navigation state
  const nav = useKioskNavigation({
    tasks: allTasks,
    onComplete,
    onDelete,
  });

  // Render based on current state
  const renderContent = () => {
    // Show feedback message (after completing or deleting)
    if (nav.feedbackMessage) {
      return (
        <div className="kiosk-feedback">
          <div className="kiosk-feedback-icon">
            {nav.feedbackMessage === 'Done!' ? '✓' :
             nav.feedbackMessage === 'Deleted' ? '✗' : '!'}
          </div>
          <div className="kiosk-feedback-text">
            {nav.feedbackMessage}
          </div>
        </div>
      );
    }

    // Show loading during async operations
    if (nav.state === 'COMPLETING' && nav.isLoading) {
      return (
        <div className="kiosk-loading">
          <div className="kiosk-loading-spinner" />
          <div className="kiosk-loading-text">Working...</div>
        </div>
      );
    }

    // No tasks
    if (allTasks.length === 0) {
      return (
        <div className="kiosk-empty">
          <div className="kiosk-empty-icon">✓</div>
          <div className="kiosk-empty-text">All done!</div>
          <div className="kiosk-empty-sub">No tasks</div>
        </div>
      );
    }

    // Task list view
    if (nav.state === 'TASK_LIST') {
      return (
        <KioskTaskCard
          task={nav.currentTask!}
          index={nav.taskIndex}
          total={allTasks.length}
        />
      );
    }

    // Action menu
    if (nav.state === 'TASK_ACTIONS') {
      return (
        <KioskActionMenu
          taskName={nav.currentTask!.name}
          selectedAction={nav.currentAction}
          isLoading={nav.isLoading}
        />
      );
    }

    // Delete confirmation
    if (nav.state === 'DELETE_CONFIRM') {
      return (
        <KioskConfirmDialog
          taskName={nav.currentTask!.name}
          selectedOption={nav.currentConfirm}
          isLoading={nav.isLoading}
        />
      );
    }

    return null;
  };

  return (
    <div className="kiosk-container">
      {renderContent()}
    </div>
  );
}
