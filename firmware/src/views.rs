/// View state machine for kiosk navigation
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::models::{CompletionRecord, Task};

/// Possible view states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewState {
    Dashboard,
    TaskList,
    TaskActions,
    DeleteConfirm,
    Completing,
    TaskHistory,
    Settings,
    QrCode,
    Empty,
}

/// Dashboard selectable items
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardItem {
    Overdue,   // 0
    Today,     // 1
    Week,      // 2
    Total,     // 3
    AllTasks,  // 4
    Settings,  // 5
}

const DASHBOARD_ITEMS: [DashboardItem; 6] = [
    DashboardItem::Overdue,
    DashboardItem::Today,
    DashboardItem::Week,
    DashboardItem::Total,
    DashboardItem::AllTasks,
    DashboardItem::Settings,
];

/// Action menu items
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionItem {
    Done,
    History,
    Delete,
    Back,
}

const ACTION_ITEMS: [ActionItem; 4] = [
    ActionItem::Done,
    ActionItem::History,
    ActionItem::Delete,
    ActionItem::Back,
];

impl ActionItem {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Done => "Done",
            Self::History => "History",
            Self::Delete => "Delete",
            Self::Back => "Back",
        }
    }
}

/// Settings menu items
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingItem {
    ManageTasks,
    ScreenTimeout,
    Back,
}

const SETTING_ITEMS: [SettingItem; 3] = [
    SettingItem::ManageTasks,
    SettingItem::ScreenTimeout,
    SettingItem::Back,
];

/// Current view context data
pub struct ViewContext {
    pub state: ViewState,

    // Dashboard state
    pub dashboard_index: usize,
    pub task_counts: TaskCounts,
    pub filtered_urgency: Option<String>,

    // Task list state
    pub tasks: Vec<Task>,
    pub task_index: i32,  // -1 = back option

    // Action menu state
    pub action_index: usize,

    // Delete confirmation
    pub delete_confirmed: bool,

    // Completing animation
    pub completing_progress: f32,

    // History view
    pub history: Vec<CompletionRecord>,
    pub history_index: usize,

    // Settings state
    pub setting_index: usize,
    pub screen_timeout_enabled: bool,
}

/// Task counts for dashboard
#[derive(Debug, Clone, Default)]
pub struct TaskCounts {
    pub overdue: u32,
    pub today: u32,
    pub week: u32,
    pub total: u32,
}

impl ViewContext {
    pub fn new() -> Self {
        Self {
            state: ViewState::Dashboard,
            dashboard_index: 0,
            task_counts: TaskCounts::default(),
            filtered_urgency: None,
            tasks: Vec::new(),
            task_index: 0,
            action_index: 0,
            delete_confirmed: false,
            completing_progress: 0.0,
            history: Vec::new(),
            history_index: 0,
            setting_index: 0,
            screen_timeout_enabled: true,
        }
    }

    /// Get currently selected task
    pub fn current_task(&self) -> Option<&Task> {
        if self.task_index >= 0 && (self.task_index as usize) < self.tasks.len() {
            Some(&self.tasks[self.task_index as usize])
        } else {
            None
        }
    }

    /// Get currently selected dashboard item
    pub fn current_dashboard_item(&self) -> Option<DashboardItem> {
        DASHBOARD_ITEMS.get(self.dashboard_index).copied()
    }
}

/// Render command - type-safe replacement for Python dict render data
pub enum RenderCommand {
    Dashboard {
        counts: TaskCounts,
        selected: usize,
    },
    TaskCard {
        task_index: usize,
        total: usize,
        filtered: Option<String>,
    },
    BackCard {
        total: usize,
    },
    EmptyFiltered {
        filter_name: String,
    },
    Empty,
    ActionMenu {
        task_name: String,
        selected: usize,
        options: Vec<String>,
    },
    ConfirmDialog {
        task_name: String,
        confirm_selected: bool,
    },
    Completing {
        task_name: String,
        progress: f32,
    },
    History {
        task_name: String,
        selected: usize,
    },
    Settings {
        selected: usize,
        screen_timeout_enabled: bool,
    },
    QrCode,
}

/// Handles navigation between views based on encoder input
pub struct ViewNavigator {
    pub ctx: ViewContext,
}

impl ViewNavigator {
    pub fn new() -> Self {
        Self {
            ctx: ViewContext::new(),
        }
    }

    /// Update task list
    pub fn set_tasks(&mut self, tasks: Vec<Task>) {
        let len = tasks.len();
        self.ctx.tasks = tasks;

        // Clamp index
        if len > 0 && self.ctx.task_index >= len as i32 {
            self.ctx.task_index = len as i32 - 1;
        }
    }

    /// Update task counts for dashboard
    pub fn set_task_counts(&mut self, counts: TaskCounts) {
        self.ctx.task_counts = counts;
    }

    /// Update history for current task
    pub fn set_history(&mut self, history: Vec<CompletionRecord>) {
        self.ctx.history = history;
        self.ctx.history_index = 0;
    }

    /// Handle clockwise encoder rotation (scroll down)
    pub fn handle_clockwise(&mut self) {
        let ctx = &mut self.ctx;

        match ctx.state {
            ViewState::Dashboard => {
                ctx.dashboard_index = (ctx.dashboard_index + 1) % DASHBOARD_ITEMS.len();
            }
            ViewState::TaskList => {
                if !ctx.tasks.is_empty() {
                    let len = ctx.tasks.len() as i32;
                    if ctx.task_index == -1 {
                        ctx.task_index = 0; // Back -> first task
                    } else if ctx.task_index == len - 1 {
                        ctx.task_index = -1; // Last task -> back
                    } else {
                        ctx.task_index += 1;
                    }
                }
            }
            ViewState::TaskActions => {
                ctx.action_index = (ctx.action_index + 1) % ACTION_ITEMS.len();
            }
            ViewState::DeleteConfirm => {
                ctx.delete_confirmed = !ctx.delete_confirmed;
            }
            ViewState::TaskHistory => {
                if !ctx.history.is_empty() {
                    ctx.history_index = (ctx.history_index + 1).min(ctx.history.len() - 1);
                }
            }
            ViewState::Settings => {
                let max_idx = SETTING_ITEMS.len() - 1;
                ctx.setting_index = (ctx.setting_index + 1).min(max_idx);
            }
            _ => {}
        }
    }

    /// Handle counter-clockwise encoder rotation (scroll up)
    pub fn handle_counter_clockwise(&mut self) {
        let ctx = &mut self.ctx;

        match ctx.state {
            ViewState::Dashboard => {
                ctx.dashboard_index = if ctx.dashboard_index == 0 {
                    DASHBOARD_ITEMS.len() - 1
                } else {
                    ctx.dashboard_index - 1
                };
            }
            ViewState::TaskList => {
                if !ctx.tasks.is_empty() {
                    let len = ctx.tasks.len() as i32;
                    if ctx.task_index == -1 {
                        ctx.task_index = len - 1; // Back -> last task
                    } else if ctx.task_index == 0 {
                        ctx.task_index = -1; // First task -> back
                    } else {
                        ctx.task_index -= 1;
                    }
                }
            }
            ViewState::TaskActions => {
                ctx.action_index = if ctx.action_index == 0 {
                    ACTION_ITEMS.len() - 1
                } else {
                    ctx.action_index - 1
                };
            }
            ViewState::DeleteConfirm => {
                ctx.delete_confirmed = !ctx.delete_confirmed;
            }
            ViewState::TaskHistory => {
                ctx.history_index = ctx.history_index.saturating_sub(1);
            }
            ViewState::Settings => {
                ctx.setting_index = ctx.setting_index.saturating_sub(1);
            }
            _ => {}
        }
    }

    /// Handle short press (select)
    /// Returns action string if an action should be taken
    pub fn handle_press(&mut self) -> Option<&'static str> {
        let ctx = &mut self.ctx;

        match ctx.state {
            ViewState::Dashboard => {
                let item = ctx.current_dashboard_item()?;
                match item {
                    DashboardItem::AllTasks => {
                        ctx.filtered_urgency = None;
                        ctx.task_index = 0;
                        ctx.state = ViewState::TaskList;
                        return Some("show_all_tasks");
                    }
                    DashboardItem::Settings => {
                        ctx.setting_index = 0;
                        ctx.state = ViewState::Settings;
                        return Some("show_settings");
                    }
                    DashboardItem::Overdue | DashboardItem::Today | DashboardItem::Week | DashboardItem::Total => {
                        let filter = match item {
                            DashboardItem::Overdue => "overdue",
                            DashboardItem::Today => "today",
                            DashboardItem::Week => "week",
                            DashboardItem::Total => "total",
                            _ => unreachable!(),
                        };
                        ctx.filtered_urgency = Some(String::from(filter));
                        ctx.task_index = 0;
                        ctx.state = ViewState::TaskList;
                        return Some("filter_tasks");
                    }
                }
            }
            ViewState::TaskList => {
                if ctx.task_index == -1 {
                    // Back selected
                    ctx.filtered_urgency = None;
                    ctx.task_index = 0;
                    ctx.state = ViewState::Dashboard;
                    return Some("go_dashboard");
                } else if !ctx.tasks.is_empty() {
                    ctx.action_index = 0;
                    ctx.state = ViewState::TaskActions;
                }
            }
            ViewState::TaskActions => {
                let action = ACTION_ITEMS[ctx.action_index];
                match action {
                    ActionItem::Done => {
                        ctx.completing_progress = 0.0;
                        ctx.state = ViewState::Completing;
                        return Some("complete");
                    }
                    ActionItem::History => {
                        ctx.history_index = 0;
                        ctx.state = ViewState::TaskHistory;
                        return Some("load_history");
                    }
                    ActionItem::Delete => {
                        ctx.delete_confirmed = false;
                        ctx.state = ViewState::DeleteConfirm;
                    }
                    ActionItem::Back => {
                        ctx.state = ViewState::TaskList;
                    }
                }
            }
            ViewState::DeleteConfirm => {
                if ctx.delete_confirmed {
                    ctx.state = ViewState::TaskList;
                    return Some("delete");
                } else {
                    ctx.state = ViewState::TaskActions;
                }
            }
            ViewState::TaskHistory => {
                ctx.state = ViewState::TaskActions;
            }
            ViewState::Settings => {
                let setting = SETTING_ITEMS[ctx.setting_index];
                match setting {
                    SettingItem::ManageTasks => {
                        ctx.state = ViewState::QrCode;
                        return Some("show_qr");
                    }
                    SettingItem::ScreenTimeout => {
                        ctx.screen_timeout_enabled = !ctx.screen_timeout_enabled;
                        return Some("toggle_timeout");
                    }
                    SettingItem::Back => {
                        ctx.state = ViewState::Dashboard;
                    }
                }
            }
            _ => {}
        }

        None
    }

    /// Handle long press (back/escape)
    pub fn handle_long_press(&mut self) -> Option<&'static str> {
        let ctx = &mut self.ctx;

        match ctx.state {
            ViewState::Dashboard => {
                // Already at home
            }
            ViewState::TaskList => {
                ctx.filtered_urgency = None;
                ctx.state = ViewState::Dashboard;
                return Some("go_dashboard");
            }
            ViewState::QrCode => {
                ctx.state = ViewState::Settings;
            }
            ViewState::Settings => {
                ctx.state = ViewState::Dashboard;
                return Some("go_dashboard");
            }
            ViewState::TaskActions | ViewState::DeleteConfirm | ViewState::TaskHistory => {
                ctx.state = ViewState::TaskList;
            }
            ViewState::Completing => {
                // Can't cancel completion
            }
            ViewState::Empty => {}
        }

        None
    }

    /// Called when completion animation finishes
    pub fn complete_animation_done(&mut self) {
        self.ctx.state = ViewState::TaskList;
    }

    /// Get render command for current view
    pub fn get_render_command(&self) -> RenderCommand {
        let ctx = &self.ctx;

        match ctx.state {
            ViewState::Dashboard => RenderCommand::Dashboard {
                counts: ctx.task_counts.clone(),
                selected: ctx.dashboard_index,
            },
            ViewState::TaskList => {
                if ctx.task_index == -1 {
                    RenderCommand::BackCard {
                        total: ctx.tasks.len(),
                    }
                } else if let Some(_task) = ctx.current_task() {
                    RenderCommand::TaskCard {
                        task_index: ctx.task_index as usize,
                        total: ctx.tasks.len(),
                        filtered: ctx.filtered_urgency.clone(),
                    }
                } else if let Some(ref filtered) = ctx.filtered_urgency {
                    RenderCommand::EmptyFiltered {
                        filter_name: filtered.clone(),
                    }
                } else {
                    RenderCommand::Empty
                }
            }
            ViewState::TaskActions => {
                let task_name = ctx
                    .current_task()
                    .map(|t| t.name.clone())
                    .unwrap_or_default();
                RenderCommand::ActionMenu {
                    task_name,
                    selected: ctx.action_index,
                    options: ACTION_ITEMS.iter().map(|a| String::from(a.label())).collect(),
                }
            }
            ViewState::DeleteConfirm => {
                let task_name = ctx
                    .current_task()
                    .map(|t| t.name.clone())
                    .unwrap_or_default();
                RenderCommand::ConfirmDialog {
                    task_name,
                    confirm_selected: ctx.delete_confirmed,
                }
            }
            ViewState::Completing => {
                let task_name = ctx
                    .current_task()
                    .map(|t| t.name.clone())
                    .unwrap_or_default();
                RenderCommand::Completing {
                    task_name,
                    progress: ctx.completing_progress,
                }
            }
            ViewState::TaskHistory => {
                let task_name = ctx
                    .current_task()
                    .map(|t| t.name.clone())
                    .unwrap_or_default();
                RenderCommand::History {
                    task_name,
                    selected: ctx.history_index,
                }
            }
            ViewState::Settings => RenderCommand::Settings {
                selected: ctx.setting_index,
                screen_timeout_enabled: ctx.screen_timeout_enabled,
            },
            ViewState::QrCode => RenderCommand::QrCode,
            ViewState::Empty => RenderCommand::Empty,
        }
    }
}
