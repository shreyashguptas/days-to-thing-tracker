-- CreateTable
CREATE TABLE "TaskCompletion" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "taskId" TEXT NOT NULL,
    "completedAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "TaskCompletion_taskId_fkey" FOREIGN KEY ("taskId") REFERENCES "Task" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

-- CreateIndex
CREATE INDEX "TaskCompletion_taskId_idx" ON "TaskCompletion"("taskId");

-- CreateIndex
CREATE INDEX "TaskCompletion_completedAt_idx" ON "TaskCompletion"("completedAt");
