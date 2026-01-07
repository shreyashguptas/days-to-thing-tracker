'use client';

import { TaskWithDue } from '@/types';

interface KioskTaskCardProps {
  task: TaskWithDue;
  index: number;
  total: number;
}

export function KioskTaskCard({ task, index, total }: KioskTaskCardProps) {
  const { name, daysUntilDue, urgency } = task;

  // Urgency-based styling
  const urgencyConfig = {
    overdue: {
      badge: 'OVERDUE',
      badgeClass: 'kiosk-badge-overdue',
      countClass: 'kiosk-count-overdue',
    },
    today: {
      badge: 'TODAY',
      badgeClass: 'kiosk-badge-today',
      countClass: 'kiosk-count-today',
    },
    'this-week': {
      badge: 'THIS WEEK',
      badgeClass: 'kiosk-badge-week',
      countClass: 'kiosk-count-week',
    },
    upcoming: {
      badge: 'UPCOMING',
      badgeClass: 'kiosk-badge-upcoming',
      countClass: 'kiosk-count-upcoming',
    },
  };

  const config = urgencyConfig[urgency];

  // Format the day count display
  const formatDayCount = () => {
    if (urgency === 'today') {
      return { number: '!', label: 'TODAY' };
    }
    if (urgency === 'overdue') {
      return { number: String(Math.abs(daysUntilDue)), label: 'DAYS OVER' };
    }
    return { number: String(daysUntilDue), label: daysUntilDue === 1 ? 'DAY LEFT' : 'DAYS LEFT' };
  };

  const dayDisplay = formatDayCount();

  return (
    <div className="kiosk-task-card">
      {/* Urgency badge */}
      <div className={`kiosk-badge ${config.badgeClass}`}>
        {config.badge}
      </div>

      {/* Task name */}
      <div className="kiosk-task-name">
        {name}
      </div>

      {/* Day count - prominent */}
      <div className={`kiosk-day-count ${config.countClass}`}>
        <span className="kiosk-day-number">{dayDisplay.number}</span>
        <span className="kiosk-day-label">{dayDisplay.label}</span>
      </div>

      {/* Navigation hint */}
      <div className="kiosk-hint">
        {total > 1 && <span className="kiosk-position">{index + 1}/{total}</span>}
        <span className="kiosk-hint-text">↕ scroll • press</span>
      </div>
    </div>
  );
}
