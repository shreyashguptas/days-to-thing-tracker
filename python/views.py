"""
View state machine for kiosk navigation
"""
from dataclasses import dataclass, field
from enum import Enum, auto
from typing import List, Optional

from models import CompletionRecord, Task


class ViewState(Enum):
    """Possible view states"""
    TASK_LIST = auto()
    TASK_ACTIONS = auto()
    DELETE_CONFIRM = auto()
    COMPLETING = auto()
    TASK_HISTORY = auto()
    SETTINGS = auto()
    QR_CODE = auto()
    EMPTY = auto()


class ActionItem(Enum):
    """Action menu items"""
    DONE = "Done"
    HISTORY = "History"
    DELETE = "Delete"
    BACK = "Back"


class SettingItem(Enum):
    """Settings menu items"""
    MANAGE_TASKS = "Manage Tasks"
    SCREEN_TIMEOUT = "Screen Timeout"
    BACK = "Back"


@dataclass
class ViewContext:
    """Current view state and context data"""
    state: ViewState = ViewState.TASK_LIST

    # Task list state
    tasks: List[Task] = field(default_factory=list)
    task_index: int = 0

    # Action menu state
    action_index: int = 0
    actions: List[ActionItem] = field(default_factory=lambda: list(ActionItem))

    # Delete confirmation
    delete_confirmed: bool = False

    # Completing animation
    completing_progress: float = 0.0

    # History view
    history: List[CompletionRecord] = field(default_factory=list)
    history_index: int = 0

    # Settings state
    setting_index: int = 0
    screen_timeout_enabled: bool = True

    @property
    def current_task(self) -> Optional[Task]:
        """Get currently selected task"""
        if 0 <= self.task_index < len(self.tasks):
            return self.tasks[self.task_index]
        return None


class ViewNavigator:
    """Handles navigation between views based on encoder input"""

    def __init__(self):
        self.ctx = ViewContext()

    def set_tasks(self, tasks: List[Task]):
        """Update task list"""
        self.ctx.tasks = tasks

        # Handle empty state
        if not tasks:
            self.ctx.state = ViewState.EMPTY
            return

        # Clamp index
        if self.ctx.task_index >= len(tasks):
            self.ctx.task_index = len(tasks) - 1

        if self.ctx.state == ViewState.EMPTY:
            self.ctx.state = ViewState.TASK_LIST

    def set_history(self, history: List[CompletionRecord]):
        """Update history for current task"""
        self.ctx.history = history
        self.ctx.history_index = 0

    def handle_clockwise(self):
        """Handle clockwise encoder rotation (scroll down)"""
        ctx = self.ctx

        if ctx.state == ViewState.TASK_LIST:
            if ctx.tasks:
                ctx.task_index = (ctx.task_index + 1) % len(ctx.tasks)

        elif ctx.state == ViewState.TASK_ACTIONS:
            ctx.action_index = (ctx.action_index + 1) % len(ctx.actions)

        elif ctx.state == ViewState.DELETE_CONFIRM:
            ctx.delete_confirmed = not ctx.delete_confirmed

        elif ctx.state == ViewState.TASK_HISTORY:
            if ctx.history:
                ctx.history_index = min(ctx.history_index + 1, len(ctx.history) - 1)

        elif ctx.state == ViewState.SETTINGS:
            max_idx = len(SettingItem) - 1
            ctx.setting_index = min(ctx.setting_index + 1, max_idx)

    def handle_counter_clockwise(self):
        """Handle counter-clockwise encoder rotation (scroll up)"""
        ctx = self.ctx

        if ctx.state == ViewState.TASK_LIST:
            if ctx.tasks:
                ctx.task_index = (ctx.task_index - 1) % len(ctx.tasks)

        elif ctx.state == ViewState.TASK_ACTIONS:
            ctx.action_index = (ctx.action_index - 1) % len(ctx.actions)

        elif ctx.state == ViewState.DELETE_CONFIRM:
            ctx.delete_confirmed = not ctx.delete_confirmed

        elif ctx.state == ViewState.TASK_HISTORY:
            ctx.history_index = max(ctx.history_index - 1, 0)

        elif ctx.state == ViewState.SETTINGS:
            ctx.setting_index = max(ctx.setting_index - 1, 0)

    def handle_press(self) -> Optional[str]:
        """
        Handle short press (select).
        Returns action string if an action should be taken:
        - "complete": Complete the current task
        - "delete": Delete the current task
        - "toggle_timeout": Toggle screen timeout setting
        """
        ctx = self.ctx

        if ctx.state == ViewState.TASK_LIST:
            if ctx.tasks:
                ctx.action_index = 0
                ctx.state = ViewState.TASK_ACTIONS

        elif ctx.state == ViewState.TASK_ACTIONS:
            action = ctx.actions[ctx.action_index]

            if action == ActionItem.DONE:
                ctx.completing_progress = 0.0
                ctx.state = ViewState.COMPLETING
                return "complete"

            elif action == ActionItem.HISTORY:
                ctx.history_index = 0
                ctx.state = ViewState.TASK_HISTORY
                return "load_history"

            elif action == ActionItem.DELETE:
                ctx.delete_confirmed = False
                ctx.state = ViewState.DELETE_CONFIRM

            elif action == ActionItem.BACK:
                ctx.state = ViewState.TASK_LIST

        elif ctx.state == ViewState.DELETE_CONFIRM:
            if ctx.delete_confirmed:
                ctx.state = ViewState.TASK_LIST
                return "delete"
            else:
                ctx.state = ViewState.TASK_ACTIONS

        elif ctx.state == ViewState.TASK_HISTORY:
            # Press in history just goes back
            ctx.state = ViewState.TASK_ACTIONS

        elif ctx.state == ViewState.SETTINGS:
            settings = list(SettingItem)
            setting = settings[ctx.setting_index]

            if setting == SettingItem.MANAGE_TASKS:
                ctx.state = ViewState.QR_CODE
                return "show_qr"

            elif setting == SettingItem.SCREEN_TIMEOUT:
                ctx.screen_timeout_enabled = not ctx.screen_timeout_enabled
                return "toggle_timeout"

            elif setting == SettingItem.BACK:
                ctx.state = ViewState.TASK_LIST

        return None

    def handle_long_press(self) -> Optional[str]:
        """Handle long press (back/escape or settings)"""
        ctx = self.ctx

        if ctx.state == ViewState.TASK_LIST:
            # Long press on task list opens settings
            ctx.setting_index = 0
            ctx.state = ViewState.SETTINGS

        elif ctx.state == ViewState.QR_CODE:
            # Go back to settings from QR code
            ctx.state = ViewState.SETTINGS

        elif ctx.state in (ViewState.TASK_ACTIONS, ViewState.DELETE_CONFIRM,
                          ViewState.TASK_HISTORY, ViewState.SETTINGS):
            # Go back to task list
            ctx.state = ViewState.TASK_LIST

        elif ctx.state == ViewState.COMPLETING:
            # Can't cancel completion
            pass

        return None

    def complete_animation_done(self):
        """Called when completion animation finishes"""
        self.ctx.state = ViewState.TASK_LIST

    def get_render_data(self) -> dict:
        """Get data needed for rendering current view"""
        ctx = self.ctx

        base = {
            "state": ctx.state.name,
        }

        if ctx.state == ViewState.TASK_LIST and ctx.current_task:
            base.update({
                "task": ctx.current_task.to_display_dict(),
                "index": ctx.task_index,
                "total": len(ctx.tasks),
            })

        elif ctx.state == ViewState.TASK_ACTIONS and ctx.current_task:
            base.update({
                "task_name": ctx.current_task.name,
                "selected": ctx.action_index,
                "options": [a.value for a in ctx.actions],
            })

        elif ctx.state == ViewState.DELETE_CONFIRM and ctx.current_task:
            base.update({
                "task_name": ctx.current_task.name,
                "confirm_selected": ctx.delete_confirmed,
            })

        elif ctx.state == ViewState.COMPLETING and ctx.current_task:
            base.update({
                "task_name": ctx.current_task.name,
                "progress": ctx.completing_progress,
            })

        elif ctx.state == ViewState.TASK_HISTORY and ctx.current_task:
            base.update({
                "task_name": ctx.current_task.name,
                "entries": [h.to_display_dict() for h in ctx.history],
                "selected": ctx.history_index,
            })

        elif ctx.state == ViewState.SETTINGS:
            base.update({
                "selected": ctx.setting_index,
                "screen_timeout_enabled": ctx.screen_timeout_enabled,
            })

        return base
