import { NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import {
  calculateNextDueDate,
  calculateDaysUntilDue,
  calculateUrgency,
} from "@/lib/date-utils";
import type {
  IntervalUnit,
  TaskWithDue,
  TasksResponse,
  CreateTaskInput,
} from "@/types";

export async function GET() {
  try {
    const tasks = await prisma.task.findMany({
      where: { isArchived: false },
      orderBy: { createdAt: "desc" },
    });

    const tasksWithDue: TaskWithDue[] = tasks.map((task) => {
      const nextDueDate = calculateNextDueDate(
        task.lastCompletedAt,
        task.intervalValue,
        task.intervalUnit as IntervalUnit,
        task.createdAt
      );
      const daysUntilDue = calculateDaysUntilDue(nextDueDate);
      const urgency = calculateUrgency(daysUntilDue);

      return {
        id: task.id,
        name: task.name,
        description: task.description,
        intervalValue: task.intervalValue,
        intervalUnit: task.intervalUnit as IntervalUnit,
        lastCompletedAt: task.lastCompletedAt?.toISOString() ?? null,
        createdAt: task.createdAt.toISOString(),
        updatedAt: task.updatedAt.toISOString(),
        isArchived: task.isArchived,
        nextDueDate: nextDueDate.toISOString(),
        daysUntilDue,
        urgency,
      };
    });

    const sorted = tasksWithDue.sort(
      (a, b) =>
        new Date(a.nextDueDate).getTime() - new Date(b.nextDueDate).getTime()
    );

    const response: TasksResponse = {
      overdue: sorted.filter((t) => t.urgency === "overdue"),
      today: sorted.filter((t) => t.urgency === "today"),
      thisWeek: sorted.filter((t) => t.urgency === "this-week"),
      upcoming: sorted.filter((t) => t.urgency === "upcoming"),
    };

    return NextResponse.json(response);
  } catch (error) {
    console.error("Failed to fetch tasks:", error);
    return NextResponse.json(
      { error: "Failed to fetch tasks" },
      { status: 500 }
    );
  }
}

export async function POST(request: Request) {
  try {
    const body: CreateTaskInput = await request.json();

    if (!body.name || !body.intervalValue || !body.intervalUnit) {
      return NextResponse.json(
        { error: "Missing required fields: name, intervalValue, intervalUnit" },
        { status: 400 }
      );
    }

    if (!["days", "weeks", "months"].includes(body.intervalUnit)) {
      return NextResponse.json(
        { error: "intervalUnit must be 'days', 'weeks', or 'months'" },
        { status: 400 }
      );
    }

    if (body.intervalValue < 1) {
      return NextResponse.json(
        { error: "intervalValue must be at least 1" },
        { status: 400 }
      );
    }

    const task = await prisma.task.create({
      data: {
        name: body.name,
        description: body.description ?? null,
        intervalValue: body.intervalValue,
        intervalUnit: body.intervalUnit,
      },
    });

    return NextResponse.json(task, { status: 201 });
  } catch (error) {
    console.error("Failed to create task:", error);
    return NextResponse.json(
      { error: "Failed to create task" },
      { status: 500 }
    );
  }
}
