import { NextResponse } from "next/server";
import { prisma } from "@/lib/prisma";

export async function GET(
  request: Request,
  { params }: { params: Promise<{ id: string }> }
) {
  try {
    const { id } = await params;

    const completions = await prisma.taskCompletion.findMany({
      where: { taskId: id },
      orderBy: { completedAt: "desc" },
      take: 50, // Limit to last 50 completions
    });

    return NextResponse.json({
      completions: completions.map((c) => ({
        id: c.id,
        completedAt: c.completedAt.toISOString(),
      })),
    });
  } catch (error) {
    console.error("Failed to fetch task history:", error);
    return NextResponse.json(
      { error: "Failed to fetch task history" },
      { status: 500 }
    );
  }
}
