CREATE TABLE "messages" (
    "id" TEXT PRIMARY KEY NOT NULL,
    "stanza_id" TEXT,
    "target" TEXT,
    "to" TEXT NOT NULL,
    "from" TEXT NOT NULL,
    "timestamp" DATETIME NOT NULL,
    "payload" TEXT NOT NULL,
    "is_first_message" BOOLEAN NOT NULL
);
CREATE INDEX "target_idx" ON "messages"("target");
CREATE INDEX "stanza_id_idx" ON "messages"("stanza_id");
CREATE INDEX "timestamp_idx" ON "messages"("timestamp");