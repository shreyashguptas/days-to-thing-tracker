"""
Data models for Days Tracker
"""
from dataclasses import dataclass
from datetime import date, datetime
from enum import Enum
from typing import Optional


class RecurrenceType(Enum):
    """Task recurrence patterns"""
    DAILY = "daily"
    WEEKLY = "weekly"
    MONTHLY = "monthly"
    YEARLY = "yearly"


class Urgency(Enum):
    """Task urgency levels based on days until due"""
    OVERDUE = "overdue"
    TODAY = "today"
    TOMORROW = "tomorrow"
    WEEK = "week"
    UPCOMING = "upcoming"

    @classmethod
    def from_days(cls, days: int) -> "Urgency":
        """Determine urgency from days until due"""
        if days < 0:
            return cls.OVERDUE
        elif days == 0:
            return cls.TODAY
        elif days == 1:
            return cls.TOMORROW
        elif days <= 7:
            return cls.WEEK
        else:
            return cls.UPCOMING


@dataclass
class Task:
    """A recurring task"""
    id: int
    name: str
    recurrence_type: RecurrenceType
    recurrence_value: int  # e.g., 30 for "every 30 days"
    next_due_date: date
    created_at: datetime
    updated_at: datetime

    @property
    def days_until_due(self) -> int:
        """Calculate days until task is due"""
        today = date.today()
        delta = self.next_due_date - today
        return delta.days

    @property
    def urgency(self) -> Urgency:
        """Get urgency level"""
        return Urgency.from_days(self.days_until_due)

    def formatted_due_date(self) -> str:
        """Format due date for display (e.g., 'Jan 15, 2026')"""
        return self.next_due_date.strftime("%b %d, %Y")

    def to_display_dict(self) -> dict:
        """Convert to dictionary for display rendering"""
        return {
            "id": self.id,
            "name": self.name,
            "days_until_due": self.days_until_due,
            "urgency": self.urgency.value,
            "next_due_date": self.formatted_due_date(),
        }


@dataclass
class CompletionRecord:
    """Record of task completion"""
    id: int
    task_id: int
    completed_at: datetime
    days_since_last: Optional[int]

    def formatted_date(self) -> str:
        """Format completion date for display"""
        return self.completed_at.strftime("%b %d, %Y")

    def to_display_dict(self) -> dict:
        """Convert to dictionary for display"""
        return {
            "completed_at": self.formatted_date(),
            "days_since_last": self.days_since_last,
        }
