"""
SQLite database operations for Days Tracker
"""
import sqlite3
from contextlib import contextmanager
from datetime import date, datetime, timedelta
from pathlib import Path
from typing import Generator, List, Optional

from models import CompletionRecord, RecurrenceType, Task


class Database:
    """SQLite database for task storage"""

    def __init__(self, db_path: Path):
        self.db_path = db_path
        self._init_db()

    def _init_db(self):
        """Initialize database schema"""
        with self._connection() as conn:
            conn.executescript("""
                CREATE TABLE IF NOT EXISTS tasks (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    recurrence_type TEXT NOT NULL,
                    recurrence_value INTEGER NOT NULL,
                    next_due_date TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS completion_history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    task_id INTEGER NOT NULL,
                    completed_at TEXT NOT NULL,
                    days_since_last INTEGER,
                    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
                );

                CREATE INDEX IF NOT EXISTS idx_tasks_due ON tasks(next_due_date);
                CREATE INDEX IF NOT EXISTS idx_history_task ON completion_history(task_id);
            """)

    @contextmanager
    def _connection(self) -> Generator[sqlite3.Connection, None, None]:
        """Context manager for database connections"""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        conn.execute("PRAGMA foreign_keys = ON")
        try:
            yield conn
            conn.commit()
        except Exception:
            conn.rollback()
            raise
        finally:
            conn.close()

    def _row_to_task(self, row: sqlite3.Row) -> Task:
        """Convert database row to Task object"""
        return Task(
            id=row["id"],
            name=row["name"],
            recurrence_type=RecurrenceType(row["recurrence_type"]),
            recurrence_value=row["recurrence_value"],
            next_due_date=date.fromisoformat(row["next_due_date"]),
            created_at=datetime.fromisoformat(row["created_at"]),
            updated_at=datetime.fromisoformat(row["updated_at"]),
        )

    def _row_to_completion(self, row: sqlite3.Row) -> CompletionRecord:
        """Convert database row to CompletionRecord"""
        return CompletionRecord(
            id=row["id"],
            task_id=row["task_id"],
            completed_at=datetime.fromisoformat(row["completed_at"]),
            days_since_last=row["days_since_last"],
        )

    def get_all_tasks(self, sort_by_due: bool = True) -> List[Task]:
        """Get all tasks, optionally sorted by due date"""
        with self._connection() as conn:
            order = "next_due_date ASC" if sort_by_due else "name ASC"
            cursor = conn.execute(f"SELECT * FROM tasks ORDER BY {order}")
            return [self._row_to_task(row) for row in cursor.fetchall()]

    def get_task(self, task_id: int) -> Optional[Task]:
        """Get a single task by ID"""
        with self._connection() as conn:
            cursor = conn.execute("SELECT * FROM tasks WHERE id = ?", (task_id,))
            row = cursor.fetchone()
            return self._row_to_task(row) if row else None

    def create_task(
        self,
        name: str,
        recurrence_type: RecurrenceType,
        recurrence_value: int,
        next_due_date: date,
    ) -> Task:
        """Create a new task"""
        now = datetime.now().isoformat()
        with self._connection() as conn:
            cursor = conn.execute(
                """
                INSERT INTO tasks (name, recurrence_type, recurrence_value, next_due_date, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?)
                """,
                (name, recurrence_type.value, recurrence_value, next_due_date.isoformat(), now, now),
            )
            return self.get_task(cursor.lastrowid)

    def update_task(
        self,
        task_id: int,
        name: Optional[str] = None,
        recurrence_type: Optional[RecurrenceType] = None,
        recurrence_value: Optional[int] = None,
        next_due_date: Optional[date] = None,
    ) -> Optional[Task]:
        """Update an existing task"""
        task = self.get_task(task_id)
        if not task:
            return None

        with self._connection() as conn:
            conn.execute(
                """
                UPDATE tasks SET
                    name = COALESCE(?, name),
                    recurrence_type = COALESCE(?, recurrence_type),
                    recurrence_value = COALESCE(?, recurrence_value),
                    next_due_date = COALESCE(?, next_due_date),
                    updated_at = ?
                WHERE id = ?
                """,
                (
                    name,
                    recurrence_type.value if recurrence_type else None,
                    recurrence_value,
                    next_due_date.isoformat() if next_due_date else None,
                    datetime.now().isoformat(),
                    task_id,
                ),
            )
        return self.get_task(task_id)

    def delete_task(self, task_id: int) -> bool:
        """Delete a task and its history"""
        with self._connection() as conn:
            cursor = conn.execute("DELETE FROM tasks WHERE id = ?", (task_id,))
            return cursor.rowcount > 0

    def complete_task(self, task_id: int) -> Optional[Task]:
        """Mark a task as completed and update next due date"""
        task = self.get_task(task_id)
        if not task:
            return None

        now = datetime.now()

        # Calculate days since last completion
        last_completion = self.get_last_completion(task_id)
        days_since_last = None
        if last_completion:
            delta = now - last_completion.completed_at
            days_since_last = delta.days

        # Record completion
        with self._connection() as conn:
            conn.execute(
                """
                INSERT INTO completion_history (task_id, completed_at, days_since_last)
                VALUES (?, ?, ?)
                """,
                (task_id, now.isoformat(), days_since_last),
            )

        # Calculate next due date from the PREVIOUS due date (fixed schedule)
        # This maintains the original cycle rather than resetting from today
        next_due = self._calculate_next_due_from_date(
            task.next_due_date,
            task.recurrence_type,
            task.recurrence_value
        )

        return self.update_task(task_id, next_due_date=next_due)

    def _calculate_next_due_from_date(
        self, from_date: date, recurrence_type: RecurrenceType, value: int
    ) -> date:
        """Calculate next due date from a given date based on recurrence"""
        if recurrence_type == RecurrenceType.DAILY:
            return from_date + timedelta(days=value)
        elif recurrence_type == RecurrenceType.WEEKLY:
            return from_date + timedelta(weeks=value)
        elif recurrence_type == RecurrenceType.MONTHLY:
            # Add months (approximate with 30 days)
            return from_date + timedelta(days=value * 30)
        elif recurrence_type == RecurrenceType.YEARLY:
            return from_date + timedelta(days=value * 365)

        return from_date + timedelta(days=value)

    def get_task_history(self, task_id: int, limit: int = 50) -> List[CompletionRecord]:
        """Get completion history for a task"""
        with self._connection() as conn:
            cursor = conn.execute(
                """
                SELECT * FROM completion_history
                WHERE task_id = ?
                ORDER BY completed_at DESC
                LIMIT ?
                """,
                (task_id, limit),
            )
            return [self._row_to_completion(row) for row in cursor.fetchall()]

    def get_last_completion(self, task_id: int) -> Optional[CompletionRecord]:
        """Get most recent completion for a task"""
        history = self.get_task_history(task_id, limit=1)
        return history[0] if history else None
