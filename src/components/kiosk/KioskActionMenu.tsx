'use client';

import { TaskAction } from '@/hooks/useKioskNavigation';

interface KioskActionMenuProps {
  taskName: string;
  selectedAction: TaskAction;
  isLoading: boolean;
}

const actionConfig: Record<TaskAction, { icon: string; label: string; className: string }> = {
  done: {
    icon: '✓',
    label: 'DONE',
    className: 'kiosk-action-done',
  },
  delete: {
    icon: '✗',
    label: 'DELETE',
    className: 'kiosk-action-delete',
  },
  back: {
    icon: '←',
    label: 'BACK',
    className: 'kiosk-action-back',
  },
};

const actions: TaskAction[] = ['done', 'delete', 'back'];

export function KioskActionMenu({
  taskName,
  selectedAction,
  isLoading,
}: KioskActionMenuProps) {
  return (
    <div className="kiosk-action-menu">
      {/* Task name header */}
      <div className="kiosk-action-header">
        {taskName}
      </div>

      {/* Divider */}
      <div className="kiosk-divider" />

      {/* Action options */}
      <div className="kiosk-actions">
        {actions.map((action) => {
          const config = actionConfig[action];
          const isSelected = action === selectedAction;
          const isDisabled = isLoading && action !== selectedAction;

          return (
            <div
              key={action}
              className={`kiosk-action-item ${config.className} ${
                isSelected ? 'kiosk-action-selected' : ''
              } ${isDisabled ? 'kiosk-action-disabled' : ''}`}
            >
              <span className="kiosk-action-indicator">
                {isSelected ? '▶' : ' '}
              </span>
              <span className="kiosk-action-icon">{config.icon}</span>
              <span className="kiosk-action-label">{config.label}</span>
            </div>
          );
        })}
      </div>

      {/* Hint */}
      <div className="kiosk-hint">
        <span className="kiosk-hint-text">↕ select • press</span>
      </div>
    </div>
  );
}
