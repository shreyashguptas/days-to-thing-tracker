#!/usr/bin/env python3
"""
REST API for remote task management

Allows adding/editing tasks from a web browser or mobile device.
Runs alongside the kiosk display.

Usage:
    python api.py
"""
from datetime import date, datetime
from flask import Flask, jsonify, request
from flask_cors import CORS

import config
from database import Database
from models import RecurrenceType

app = Flask(__name__)
CORS(app)

db = Database(config.DATABASE_PATH)


@app.route("/api/tasks", methods=["GET"])
def get_tasks():
    """Get all tasks"""
    tasks = db.get_all_tasks()
    return jsonify([
        {
            "id": t.id,
            "name": t.name,
            "recurrenceType": t.recurrence_type.value,
            "recurrenceValue": t.recurrence_value,
            "nextDueDate": t.next_due_date.isoformat(),
            "daysUntilDue": t.days_until_due,
            "urgency": t.urgency.value,
            "createdAt": t.created_at.isoformat(),
            "updatedAt": t.updated_at.isoformat(),
        }
        for t in tasks
    ])


@app.route("/api/tasks", methods=["POST"])
def create_task():
    """Create a new task"""
    data = request.json

    try:
        name = data["name"]
        recurrence_type = RecurrenceType(data["recurrenceType"])
        recurrence_value = int(data["recurrenceValue"])
        next_due_date = date.fromisoformat(data["nextDueDate"])

        task = db.create_task(name, recurrence_type, recurrence_value, next_due_date)

        return jsonify({
            "id": task.id,
            "name": task.name,
            "recurrenceType": task.recurrence_type.value,
            "recurrenceValue": task.recurrence_value,
            "nextDueDate": task.next_due_date.isoformat(),
            "daysUntilDue": task.days_until_due,
            "urgency": task.urgency.value,
        }), 201

    except (KeyError, ValueError) as e:
        return jsonify({"error": str(e)}), 400


@app.route("/api/tasks/<int:task_id>", methods=["GET"])
def get_task(task_id: int):
    """Get a single task"""
    task = db.get_task(task_id)
    if not task:
        return jsonify({"error": "Task not found"}), 404

    return jsonify({
        "id": task.id,
        "name": task.name,
        "recurrenceType": task.recurrence_type.value,
        "recurrenceValue": task.recurrence_value,
        "nextDueDate": task.next_due_date.isoformat(),
        "daysUntilDue": task.days_until_due,
        "urgency": task.urgency.value,
        "createdAt": task.created_at.isoformat(),
        "updatedAt": task.updated_at.isoformat(),
    })


@app.route("/api/tasks/<int:task_id>", methods=["PUT"])
def update_task(task_id: int):
    """Update a task"""
    data = request.json

    recurrence_type = None
    if "recurrenceType" in data:
        recurrence_type = RecurrenceType(data["recurrenceType"])

    next_due_date = None
    if "nextDueDate" in data:
        next_due_date = date.fromisoformat(data["nextDueDate"])

    task = db.update_task(
        task_id,
        name=data.get("name"),
        recurrence_type=recurrence_type,
        recurrence_value=data.get("recurrenceValue"),
        next_due_date=next_due_date,
    )

    if not task:
        return jsonify({"error": "Task not found"}), 404

    return jsonify({
        "id": task.id,
        "name": task.name,
        "recurrenceType": task.recurrence_type.value,
        "recurrenceValue": task.recurrence_value,
        "nextDueDate": task.next_due_date.isoformat(),
        "daysUntilDue": task.days_until_due,
        "urgency": task.urgency.value,
    })


@app.route("/api/tasks/<int:task_id>", methods=["DELETE"])
def delete_task(task_id: int):
    """Delete a task"""
    if db.delete_task(task_id):
        return "", 204
    return jsonify({"error": "Task not found"}), 404


@app.route("/api/tasks/<int:task_id>/complete", methods=["POST"])
def complete_task(task_id: int):
    """Mark a task as complete"""
    task = db.complete_task(task_id)
    if not task:
        return jsonify({"error": "Task not found"}), 404

    return jsonify({
        "id": task.id,
        "name": task.name,
        "nextDueDate": task.next_due_date.isoformat(),
        "daysUntilDue": task.days_until_due,
        "urgency": task.urgency.value,
    })


@app.route("/api/tasks/<int:task_id>/history", methods=["GET"])
def get_task_history(task_id: int):
    """Get completion history for a task"""
    task = db.get_task(task_id)
    if not task:
        return jsonify({"error": "Task not found"}), 404

    history = db.get_task_history(task_id)

    return jsonify([
        {
            "id": h.id,
            "completedAt": h.completed_at.isoformat(),
            "daysSinceLast": h.days_since_last,
        }
        for h in history
    ])


@app.route("/health", methods=["GET"])
def health_check():
    """Health check endpoint"""
    return jsonify({"status": "ok", "timestamp": datetime.now().isoformat()})


def main():
    """Run the API server"""
    print(f"Starting API server on {config.API_HOST}:{config.API_PORT}")
    app.run(host=config.API_HOST, port=config.API_PORT, debug=False)


if __name__ == "__main__":
    main()
