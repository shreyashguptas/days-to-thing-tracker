/// Data models for Days Tracker
extern crate alloc;

use alloc::string::String;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Task recurrence patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecurrenceType {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl RecurrenceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Yearly => "yearly",
        }
    }
}

/// Task urgency levels based on days until due
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Urgency {
    Overdue,
    Today,
    Tomorrow,
    Week,
    Upcoming,
}

impl Urgency {
    /// Determine urgency from days until due
    pub fn from_days(days: i32) -> Self {
        if days < 0 {
            Self::Overdue
        } else if days == 0 {
            Self::Today
        } else if days == 1 {
            Self::Tomorrow
        } else if days <= 7 {
            Self::Week
        } else {
            Self::Upcoming
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Overdue => "overdue",
            Self::Today => "today",
            Self::Tomorrow => "tomorrow",
            Self::Week => "week",
            Self::Upcoming => "upcoming",
        }
    }
}

/// A recurring task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub name: String,
    pub recurrence_type: RecurrenceType,
    pub recurrence_value: u32,
    pub next_due_date: String,     // ISO format "YYYY-MM-DD"
    pub created_at: String,        // ISO format datetime
    pub updated_at: String,        // ISO format datetime
}

impl Task {
    /// Parse next_due_date string to NaiveDate
    pub fn due_date(&self) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(&self.next_due_date, "%Y-%m-%d").ok()
    }

    /// Calculate days until task is due
    pub fn days_until_due(&self, today: NaiveDate) -> i32 {
        match self.due_date() {
            Some(due) => (due - today).num_days() as i32,
            None => 0,
        }
    }

    /// Get urgency level
    pub fn urgency(&self, today: NaiveDate) -> Urgency {
        Urgency::from_days(self.days_until_due(today))
    }

    /// Format due date for display (e.g., "Jan 15, 2026")
    pub fn formatted_due_date(&self) -> String {
        match self.due_date() {
            Some(date) => date.format("%b %d, %Y").to_string(),
            None => self.next_due_date.clone(),
        }
    }
}

/// Record of task completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRecord {
    pub id: u32,
    pub task_id: u32,
    pub completed_at: String,     // ISO format datetime
    pub days_since_last: Option<i32>,
}

impl CompletionRecord {
    /// Format completion date for display
    pub fn formatted_date(&self) -> String {
        // Parse datetime and format as "Jan 15, 2026"
        if let Some(date) = NaiveDate::parse_from_str(
            self.completed_at.split('T').next().unwrap_or(&self.completed_at),
            "%Y-%m-%d",
        )
        .ok()
        {
            date.format("%b %d, %Y").to_string()
        } else {
            self.completed_at.clone()
        }
    }
}

/// Task data for rendering (lightweight view struct)
pub struct TaskDisplayData {
    pub name: String,
    pub days_until_due: i32,
    pub urgency: String,
    pub next_due_date: String,
}

/// History entry for rendering
pub struct HistoryDisplayEntry {
    pub completed_at: String,
    pub days_since_last: Option<i32>,
}
