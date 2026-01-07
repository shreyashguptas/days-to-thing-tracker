'use client';

import { useState, useEffect } from 'react';
import { formatLastCompleted } from '@/lib/date-utils';

interface Completion {
  id: string;
  completedAt: string;
}

interface KioskHistoryViewProps {
  taskId: string;
  taskName: string;
  selectedIndex: number;
  onHistoryLoaded: (length: number) => void;
}

export function KioskHistoryView({
  taskId,
  taskName,
  selectedIndex,
  onHistoryLoaded,
}: KioskHistoryViewProps) {
  const [completions, setCompletions] = useState<Completion[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setIsLoading(true);
    setError(null);

    fetch(`/api/tasks/${taskId}/history`)
      .then((res) => res.json())
      .then((data) => {
        if (data.error) {
          setError(data.error);
          onHistoryLoaded(0);
        } else {
          const items = data.completions || [];
          setCompletions(items);
          onHistoryLoaded(items.length);
        }
      })
      .catch(() => {
        setError('Failed');
        onHistoryLoaded(0);
      })
      .finally(() => setIsLoading(false));
  }, [taskId, onHistoryLoaded]);

  if (isLoading) {
    return (
      <div className="kiosk-loading">
        <div className="kiosk-loading-spinner" />
        <div className="kiosk-loading-text">Loading...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="kiosk-feedback">
        <div className="kiosk-feedback-icon" style={{ color: 'var(--urgency-overdue)' }}>!</div>
        <div className="kiosk-feedback-text">Error</div>
      </div>
    );
  }

  if (completions.length === 0) {
    return (
      <div className="kiosk-history">
        <div className="kiosk-history-header">HISTORY</div>
        <div className="kiosk-divider" />
        <div className="kiosk-history-empty">
          <div className="kiosk-history-empty-icon">⏱</div>
          <div className="kiosk-history-empty-text">No history</div>
        </div>
        <div className="kiosk-hint">
          <span className="kiosk-hint-text">press to go back</span>
        </div>
      </div>
    );
  }

  // Show 3 completions at a time, centered on selected
  const visibleCount = 3;
  const startIdx = Math.max(0, Math.min(selectedIndex - 1, completions.length - visibleCount));
  const visibleCompletions = completions.slice(startIdx, startIdx + visibleCount);

  return (
    <div className="kiosk-history">
      <div className="kiosk-history-header">
        <span className="kiosk-history-title">HISTORY</span>
        <span className="kiosk-history-count">{completions.length}</span>
      </div>
      <div className="kiosk-divider" />

      <div className="kiosk-history-list">
        {visibleCompletions.map((completion, idx) => {
          const actualIdx = startIdx + idx;
          const isSelected = actualIdx === selectedIndex;

          return (
            <div
              key={completion.id}
              className={`kiosk-history-item ${isSelected ? 'kiosk-history-selected' : ''}`}
            >
              <span className="kiosk-history-indicator">
                {isSelected ? '▶' : ' '}
              </span>
              <span className="kiosk-history-icon">✓</span>
              <span className="kiosk-history-date">
                {formatLastCompleted(completion.completedAt)}
              </span>
              {actualIdx === 0 && (
                <span className="kiosk-history-badge">NEW</span>
              )}
            </div>
          );
        })}
      </div>

      <div className="kiosk-hint">
        <span className="kiosk-position">{selectedIndex + 1}/{completions.length}</span>
        <span className="kiosk-hint-text">↕ scroll • press</span>
      </div>
    </div>
  );
}
