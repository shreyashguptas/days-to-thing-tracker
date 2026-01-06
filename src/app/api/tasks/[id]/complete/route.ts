import { NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";
import {
  calculateNextDueDate,
  calculateDaysUntilDue,
} from "@/lib/date-utils";
import type { IntervalUnit } from "@/types";

export async function POST(
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

    const now = new Date();

    // Update task and create completion history entry
    const [task] = await prisma.$transaction([
      prisma.task.update({
        where: { id },
        data: {
          lastCompletedAt: now,
        },
      }),
      prisma.taskCompletion.create({
        data: {
          taskId: id,
          completedAt: now,
        },
      }),
    ]);

    // Calculate new due date info for toast
    const nextDueDate = calculateNextDueDate(
      now,
      task.intervalValue,
      task.intervalUnit as IntervalUnit,
      task.createdAt
    );
    const daysUntilDue = calculateDaysUntilDue(nextDueDate);

    return NextResponse.json({
      success: true,
      task: {
        ...task,
        lastCompletedAt: task.lastCompletedAt?.toISOString() ?? null,
        createdAt: task.createdAt.toISOString(),
        updatedAt: task.updatedAt.toISOString(),
        nextDueDate: nextDueDate.toISOString(),
        daysUntilDue,
      },
    });
  } catch (error) {
    console.error("Failed to complete task:", error);
    return NextResponse.json(
      { error: "Failed to complete task" },
      { status: 500 }
    );
  }
}
