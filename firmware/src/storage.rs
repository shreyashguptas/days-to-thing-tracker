/// JSON-based storage on LittleFS/SPIFFS
///
/// JSON files stored on flash partition, loaded fully into RAM.
/// All data loaded into RAM; mutations flush to flash immediately.
extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::models::{CompletionRecord, RecurrenceType, Task};
use crate::views::TaskCounts;

/// Task store (loaded fully into RAM)
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TaskStore {
    pub tasks: Vec<Task>,
    pub next_id: u32,
}

/// History store (loaded fully into RAM)
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HistoryStore {
    pub records: Vec<CompletionRecord>,
    pub next_id: u32,
}

/// Combined storage
pub struct Storage {
    pub task_store: TaskStore,
    pub history_store: HistoryStore,
    tasks_path: String,
    history_path: String,
}

impl Storage {
    /// Create new storage instance, loading from files if they exist
    pub fn new(tasks_path: &str, history_path: &str) -> Self {
        let task_store = Self::load_json::<TaskStore>(tasks_path).unwrap_or_default();
        let history_store = Self::load_json::<HistoryStore>(history_path).unwrap_or_default();

        log::info!(
            "Storage loaded: {} tasks, {} history records",
            task_store.tasks.len(),
            history_store.records.len()
        );

        Self {
            task_store,
            history_store,
            tasks_path: String::from(tasks_path),
            history_path: String::from(history_path),
        }
    }

    /// Load JSON from a file
    fn load_json<T: for<'de> Deserialize<'de>>(path: &str) -> Option<T> {
        match std::fs::read_to_string(path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(data) => Some(data),
                Err(e) => {
                    log::warn!("Failed to parse {}: {}", path, e);
                    None
                }
            },
            Err(_) => {
                log::info!("File not found: {} (will create on first write)", path);
                None
            }
        }
    }

    /// Save task store to file
    fn save_tasks(&self) {
        match serde_json::to_string(&self.task_store) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&self.tasks_path, json) {
                    log::error!("Failed to save tasks: {}", e);
                }
            }
            Err(e) => log::error!("Failed to serialize tasks: {}", e),
        }
    }

    /// Save history store to file
    fn save_history(&self) {
        match serde_json::to_string(&self.history_store) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&self.history_path, json) {
                    log::error!("Failed to save history: {}", e);
                }
            }
            Err(e) => log::error!("Failed to serialize history: {}", e),
        }
    }

    // ========== TASK CRUD ==========

    /// Get all tasks, sorted by due date
    pub fn get_all_tasks(&self, sort_by_due: bool) -> Vec<Task> {
        let mut tasks = self.task_store.tasks.clone();
        if sort_by_due {
            tasks.sort_by(|a, b| a.next_due_date.cmp(&b.next_due_date));
        } else {
            tasks.sort_by(|a, b| a.name.cmp(&b.name));
        }
        tasks
    }

    /// Get a single task by ID
    pub fn get_task(&self, task_id: u32) -> Option<&Task> {
        self.task_store.tasks.iter().find(|t| t.id == task_id)
    }

    /// Create a new task
    pub fn create_task(
        &mut self,
        name: String,
        recurrence_type: RecurrenceType,
        recurrence_value: u32,
        next_due_date: String,
        now_iso: &str,
    ) -> &Task {
        let id = self.task_store.next_id;
        self.task_store.next_id += 1;

        let task = Task {
            id,
            name,
            recurrence_type,
            recurrence_value,
            next_due_date,
            created_at: String::from(now_iso),
            updated_at: String::from(now_iso),
        };

        self.task_store.tasks.push(task);
        self.save_tasks();

        self.task_store.tasks.last().unwrap()
    }

    /// Update an existing task
    pub fn update_task(
        &mut self,
        task_id: u32,
        name: Option<String>,
        recurrence_type: Option<RecurrenceType>,
        recurrence_value: Option<u32>,
        next_due_date: Option<String>,
        now_iso: &str,
    ) -> Option<&Task> {
        let task = self.task_store.tasks.iter_mut().find(|t| t.id == task_id)?;

        if let Some(n) = name {
            task.name = n;
        }
        if let Some(rt) = recurrence_type {
            task.recurrence_type = rt;
        }
        if let Some(rv) = recurrence_value {
            task.recurrence_value = rv;
        }
        if let Some(ndd) = next_due_date {
            task.next_due_date = ndd;
        }
        task.updated_at = String::from(now_iso);

        self.save_tasks();

        self.task_store.tasks.iter().find(|t| t.id == task_id)
    }

    /// Delete a task and its history
    pub fn delete_task(&mut self, task_id: u32) -> bool {
        let before = self.task_store.tasks.len();
        self.task_store.tasks.retain(|t| t.id != task_id);
        let deleted = self.task_store.tasks.len() < before;

        if deleted {
            // Also delete history
            self.history_store.records.retain(|r| r.task_id != task_id);
            self.save_tasks();
            self.save_history();
        }

        deleted
    }

    /// Mark a task as completed and update next due date
    pub fn complete_task(&mut self, task_id: u32, now_iso: &str, today: NaiveDate) -> bool {
        // Find the task
        let task = match self.task_store.tasks.iter().find(|t| t.id == task_id) {
            Some(t) => t.clone(),
            None => return false,
        };

        // Calculate days since last completion
        let last_completion = self.get_last_completion(task_id);
        let days_since_last = last_completion.and_then(|lc| {
            let lc_date = NaiveDate::parse_from_str(
                lc.completed_at.split('T').next().unwrap_or(&lc.completed_at),
                "%Y-%m-%d",
            )
            .ok()?;
            Some((today - lc_date).num_days() as i32)
        });

        // Record completion
        let record_id = self.history_store.next_id;
        self.history_store.next_id += 1;
        self.history_store.records.push(CompletionRecord {
            id: record_id,
            task_id,
            completed_at: String::from(now_iso),
            days_since_last,
        });
        self.save_history();

        // Calculate next due date from the PREVIOUS due date (fixed schedule)
        if let Some(due_date) = task.due_date() {
            let next_due = calculate_next_due(due_date, task.recurrence_type, task.recurrence_value);
            self.update_task(task_id, None, None, None, Some(next_due.format("%Y-%m-%d").to_string()), now_iso);
        }

        true
    }

    // ========== HISTORY ==========

    /// Get completion history for a task
    pub fn get_task_history(&self, task_id: u32) -> Vec<CompletionRecord> {
        let mut records: Vec<CompletionRecord> = self
            .history_store
            .records
            .iter()
            .filter(|r| r.task_id == task_id)
            .cloned()
            .collect();
        records.sort_by(|a, b| b.completed_at.cmp(&a.completed_at));
        records.truncate(50);
        records
    }

    /// Get most recent completion for a task
    pub fn get_last_completion(&self, task_id: u32) -> Option<&CompletionRecord> {
        self.history_store
            .records
            .iter()
            .filter(|r| r.task_id == task_id)
            .max_by(|a, b| a.completed_at.cmp(&b.completed_at))
    }

    // ========== AGGREGATIONS ==========

    /// Get task counts by urgency category for dashboard
    pub fn get_task_counts(&self, today: NaiveDate) -> TaskCounts {
        let tasks = self.get_all_tasks(true);
        let mut counts = TaskCounts {
            overdue: 0,
            today: 0,
            week: 0,
            total: tasks.len() as u32,
        };

        for task in &tasks {
            let days = task.days_until_due(today);
            if days < 0 {
                counts.overdue += 1;
                counts.week += 1;
            } else if days == 0 {
                counts.today += 1;
                counts.week += 1;
            } else if days <= 7 {
                counts.week += 1;
            }
        }

        counts
    }

    /// Get tasks filtered by urgency category
    pub fn get_tasks_by_urgency(&self, urgency: &str, today: NaiveDate) -> Vec<Task> {
        let tasks = self.get_all_tasks(true);
        match urgency {
            "overdue" => tasks
                .into_iter()
                .filter(|t| t.days_until_due(today) < 0)
                .collect(),
            "today" => tasks
                .into_iter()
                .filter(|t| t.days_until_due(today) == 0)
                .collect(),
            "week" => tasks
                .into_iter()
                .filter(|t| t.days_until_due(today) <= 7)
                .collect(),
            _ => tasks, // "total" or any other value returns all
        }
    }
}

/// Calculate next due date based on recurrence
fn calculate_next_due(from_date: NaiveDate, recurrence_type: RecurrenceType, value: u32) -> NaiveDate {
    match recurrence_type {
        RecurrenceType::Daily => from_date + chrono::Duration::days(value as i64),
        RecurrenceType::Weekly => from_date + chrono::Duration::weeks(value as i64),
        RecurrenceType::Monthly => from_date + chrono::Duration::days(value as i64 * 30),
        RecurrenceType::Yearly => from_date + chrono::Duration::days(value as i64 * 365),
    }
}
