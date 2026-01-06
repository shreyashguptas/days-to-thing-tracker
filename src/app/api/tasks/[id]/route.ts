import { NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import {
  calculateNextDueDate,
  calculateDaysUntilDue,
  calculateUrgency,
} from "@/lib/date-utils";
import type { IntervalUnit, TaskWithDue, UpdateTaskInput } from "@/types";

export async function GET(
  request: Request,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id } = await params;
    const task = await prisma.task.findUnique({
      where: { id },
    });

    if (!task) {
      return NextResponse.json({ error: "Task not found" }, { status: 404 });
    }

    const nextDueDate = calculateNextDueDate(
      task.lastCompletedAt,
      task.intervalValue,
      task.intervalUnit as IntervalUnit,
      task.createdAt
    );
    const daysUntilDue = calculateDaysUntilDue(nextDueDate);
    const urgency = calculateUrgency(daysUntilDue);

    const taskWithDue: TaskWithDue = {
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

    return NextResponse.json(taskWithDue);
  } catch (error) {
    console.error("Failed to fetch task:", error);
    return NextResponse.json(
      { error: "Failed to fetch task" },
      { status: 500 }
    );
  }
}

export async function PUT(
  request: Request,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id } = await params;
    const body: UpdateTaskInput = await request.json();

    const existingTask = await prisma.task.findUnique({
      where: { id },
    });

    if (!existingTask) {
      return NextResponse.json({ error: "Task not found" }, { status: 404 });
    }

    if (
      body.intervalUnit &&
      !["days", "weeks", "months"].includes(body.intervalUnit)
    ) {
      return NextResponse.json(
        { error: "intervalUnit must be 'days', 'weeks', or 'months'" },
        { status: 400 }
      );
    }

    if (body.intervalValue !== undefined && body.intervalValue < 1) {
      return NextResponse.json(
        { error: "intervalValue must be at least 1" },
        { status: 400 }
      );
    }

    const task = await prisma.task.update({
      where: { id },
      data: {
        ...(body.name !== undefined && { name: body.name }),
        ...(body.description !== undefined && { description: body.description }),
        ...(body.intervalValue !== undefined && {
          intervalValue: body.intervalValue,
        }),
        ...(body.intervalUnit !== undefined && {
          intervalUnit: body.intervalUnit,
        }),
      },
    });

    return NextResponse.json(task);
  } catch (error) {
    console.error("Failed to update task:", error);
    return NextResponse.json(
      { error: "Failed to update task" },
      { status: 500 }
    );
  }
}

export async function DELETE(
  request: Request,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id } = await params;
    const existingTask = await prisma.task.findUnique({
      where: { id },
    });

    if (!existingTask) {
      return NextResponse.json({ error: "Task not found" }, { status: 404 });
    }

    await prisma.task.update({
      where: { id },
      data: { isArchived: true },
    });

    return NextResponse.json({ success: true });
  } catch (error) {
    console.error("Failed to delete task:", error);
    return NextResponse.json(
      { error: "Failed to delete task" },
      { status: 500 }
    );
  }
}
