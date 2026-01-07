'use client';

import { ConfirmOption } from '@/hooks/useKioskNavigation';

interface KioskConfirmDialogProps {
  taskName: string;
  selectedOption: ConfirmOption;
  isLoading: boolean;
}

const optionConfig: Record<ConfirmOption, { label: string; className: string }> = {
  yes: {
    label: 'YES, DELETE',
    className: 'kiosk-confirm-yes',
  },
  no: {
    label: 'NO, CANCEL',
    className: 'kiosk-confirm-no',
  },
};

const options: ConfirmOption[] = ['yes', 'no'];

export function KioskConfirmDialog({
  taskName,
  selectedOption,
  isLoading,
}: KioskConfirmDialogProps) {
  return (
    <div className="kiosk-confirm-dialog">
      {/* Warning header */}
      <div className="kiosk-confirm-header">
        DELETE?
      </div>

      {/* Task name */}
      <div className="kiosk-confirm-task">
        &quot;{taskName}&quot;
      </div>

      {/* Divider */}
      <div className="kiosk-divider" />

      {/* Options */}
      <div className="kiosk-confirm-options">
        {options.map((option) => {
          const config = optionConfig[option];
          const isSelected = option === selectedOption;
          const isDisabled = isLoading && option !== selectedOption;

          return (
            <div
              key={option}
              className={`kiosk-confirm-item ${config.className} ${
                isSelected ? 'kiosk-confirm-selected' : ''
              } ${isDisabled ? 'kiosk-confirm-disabled' : ''}`}
            >
              <span className="kiosk-confirm-indicator">
                {isSelected ? '▶' : ' '}
              </span>
              <span className="kiosk-confirm-label">{config.label}</span>
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
