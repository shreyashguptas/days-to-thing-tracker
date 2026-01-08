#!/usr/bin/env python3
"""
Days Tracker Kiosk - Main Application

A standalone kiosk application for tracking recurring tasks.
Renders directly to framebuffer, no browser required.

Usage:
    python main.py
"""
import signal
import sys
import time
from threading import Thread

import config
from database import Database
from views import ViewNavigator, ViewState

# Try to import Rust core (may not be available during development)
try:
    import kiosk_core
    from kiosk_core import KioskController, TaskData, HistoryEntry
    RUST_AVAILABLE = True
except ImportError:
    print("Warning: kiosk_core not available, running in simulation mode")
    RUST_AVAILABLE = False
    KioskController = None
    TaskData = None
    HistoryEntry = None


class KioskApp:
    """Main kiosk application"""

    def __init__(self):
        self.running = True
        self.db = Database(config.DATABASE_PATH)
        self.nav = ViewNavigator()
        self.controller = None
        self.last_render_data = None

        # Load initial data
        self._load_counts()
        self._load_tasks()

        # Set up signal handlers
        signal.signal(signal.SIGINT, self._signal_handler)
        signal.signal(signal.SIGTERM, self._signal_handler)

    def _signal_handler(self, signum, frame):
        """Handle shutdown signals"""
        print("\nShutting down...")
        self.running = False
        if RUST_AVAILABLE:
            kiosk_core.shutdown()

    def _load_tasks(self, urgency_filter: str = None):
        """Load tasks from database, optionally filtered by urgency"""
        if urgency_filter:
            tasks = self.db.get_tasks_by_urgency(urgency_filter)
        else:
            tasks = self.db.get_all_tasks(sort_by_due=True)
        self.nav.set_tasks(tasks)

    def _load_counts(self):
        """Load task counts for dashboard"""
        counts = self.db.get_task_counts()
        self.nav.set_task_counts(counts)

    def _init_display(self):
        """Initialize display controller"""
        if not RUST_AVAILABLE:
            print("Running in simulation mode (no display)")
            return

        try:
            self.controller = KioskController(
                width=config.DISPLAY_WIDTH,
                height=config.DISPLAY_HEIGHT,
                clk_pin=config.PIN_CLK,
                dt_pin=config.PIN_DT,
                sw_pin=config.PIN_SW,
                bl_pin=config.PIN_BACKLIGHT,
            )
            print("Display initialized")
        except Exception as e:
            print(f"Failed to initialize display: {e}")
            print("Running in simulation mode")

    def _handle_event(self, event: str):
        """Handle encoder event"""
        action = None

        if event == "cw":
            self.nav.handle_clockwise()
        elif event == "ccw":
            self.nav.handle_counter_clockwise()
        elif event == "press":
            action = self.nav.handle_press()
        elif event == "long_press":
            action = self.nav.handle_long_press()

        # Handle actions
        if action == "complete":
            self._complete_current_task()
        elif action == "delete":
            self._delete_current_task()
        elif action == "load_history":
            self._load_history()
        elif action == "toggle_timeout":
            self._toggle_screen_timeout()
        elif action == "show_qr":
            pass  # QR code view handled in render
        elif action == "filter_tasks":
            # Load tasks filtered by the selected urgency
            self._load_tasks(self.nav.ctx.filtered_urgency)
        elif action == "show_all_tasks":
            # Load all tasks
            self._load_tasks()
        elif action == "show_settings":
            pass  # Settings view handled in render
        elif action == "go_dashboard":
            # Refresh counts when going back to dashboard
            self._load_counts()

    def _complete_current_task(self):
        """Complete the current task with animation"""
        task = self.nav.ctx.current_task
        if not task:
            return

        # Animate completion
        start_time = time.time()
        while time.time() - start_time < config.COMPLETING_DURATION:
            progress = (time.time() - start_time) / config.COMPLETING_DURATION
            self.nav.ctx.completing_progress = min(progress, 1.0)
            self._render()
            time.sleep(0.016)  # ~60fps

        # Actually complete in database
        self.db.complete_task(task.id)

        # Reload tasks and counts
        self._load_counts()
        self._load_tasks(self.nav.ctx.filtered_urgency)

        # Return to task list
        self.nav.complete_animation_done()

    def _delete_current_task(self):
        """Delete the current task"""
        task = self.nav.ctx.current_task
        if not task:
            return

        self.db.delete_task(task.id)
        self._load_counts()
        self._load_tasks(self.nav.ctx.filtered_urgency)

    def _load_history(self):
        """Load history for current task"""
        task = self.nav.ctx.current_task
        if not task:
            return

        history = self.db.get_task_history(task.id)
        self.nav.set_history(history)

    def _toggle_screen_timeout(self):
        """Toggle screen timeout setting"""
        # The nav context already toggled the setting
        enabled = self.nav.ctx.screen_timeout_enabled
        print(f"Screen timeout {'enabled' if enabled else 'disabled'}")

    def _render(self):
        """Render current view to display"""
        render_data = self.nav.get_render_data()

        # Skip if nothing changed
        if render_data == self.last_render_data:
            return
        self.last_render_data = render_data

        if not self.controller:
            # Simulation mode - print state
            print(f"View: {render_data['state']}")
            return

        state = render_data["state"]

        try:
            if state == "DASHBOARD":
                counts = render_data.get("counts", {})
                self.controller.render_dashboard(
                    counts.get("overdue", 0),
                    counts.get("today", 0),
                    counts.get("week", 0),
                    counts.get("total", 0),
                    render_data.get("selected", 0),
                )

            elif state == "TASK_LIST":
                if render_data.get("back_selected"):
                    # Show back card
                    self.controller.render_back_card(render_data.get("total", 0))
                else:
                    task = render_data.get("task")
                    filtered = render_data.get("filtered")
                    if task:
                        task_data = TaskData(
                            id=task["id"],
                            name=task["name"],
                            days_until_due=task["days_until_due"],
                            urgency=task["urgency"],
                            next_due_date=task["next_due_date"],
                        )
                        self.controller.render_task_card(
                            task_data,
                            render_data["index"],
                            render_data["total"],
                        )
                    elif filtered:
                        # Empty filtered list
                        self.controller.render_empty_filtered(filtered)
                    else:
                        # Empty overall list
                        self.controller.render_empty()

            elif state == "TASK_ACTIONS":
                self.controller.render_action_menu(
                    render_data["task_name"],
                    render_data["selected"],
                    render_data["options"],
                )

            elif state == "DELETE_CONFIRM":
                self.controller.render_confirm_dialog(
                    f"Delete '{render_data['task_name']}'?",
                    render_data["confirm_selected"],
                )

            elif state == "COMPLETING":
                self.controller.render_completing(
                    render_data["task_name"],
                    render_data["progress"],
                )

            elif state == "TASK_HISTORY":
                entries = [
                    HistoryEntry(
                        completed_at=e["completed_at"],
                        days_since_last=e["days_since_last"],
                    )
                    for e in render_data["entries"]
                ]
                self.controller.render_history(
                    render_data["task_name"],
                    entries,
                    render_data["selected"],
                )

            elif state == "SETTINGS":
                self.controller.render_settings(
                    render_data["selected"],
                    render_data["screen_timeout_enabled"],
                )

            elif state == "QR_CODE":
                self.controller.render_qr_code(config.WEB_URL)

            elif state == "EMPTY":
                self.controller.render_empty()

        except Exception as e:
            print(f"Render error: {e}")

    def _check_idle_timeout(self):
        """Check and handle screen idle timeout"""
        if not self.controller or not self.nav.ctx.screen_timeout_enabled:
            return

        idle_seconds = self.controller.seconds_since_activity()

        if idle_seconds > config.IDLE_TIMEOUT and self.controller.is_backlight_on():
            self.controller.backlight_off()
            print("Screen off (idle timeout)")

    def run(self):
        """Main application loop"""
        print("Days Tracker Kiosk Starting...")
        print(f"Database: {config.DATABASE_PATH}")

        self._init_display()
        self._render()

        print("Ready. Press Ctrl+C to exit.")

        last_idle_check = time.time()

        while self.running:
            # Poll for encoder events
            if self.controller:
                event = self.controller.poll_encoder()
                if event:
                    self._handle_event(event)
                    self._render()

            # Check idle timeout periodically
            now = time.time()
            if now - last_idle_check > 1.0:
                self._check_idle_timeout()
                last_idle_check = now

            # Small sleep to prevent CPU spin
            time.sleep(config.POLL_INTERVAL)

        print("Goodbye!")


def main():
    """Entry point"""
    app = KioskApp()
    app.run()


if __name__ == "__main__":
    main()
