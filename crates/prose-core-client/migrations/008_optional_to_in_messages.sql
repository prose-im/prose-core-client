-- MUC messages don't necessarily have a `to` property. This change reflects that…

DROP TABLE "messages";

CREATE TABLE "messages" (
    "id" TEXT PRIMARY KEY NOT NULL,
    "stanza_id" TEXT,
    "target" TEXT,
    "to" TEXT,
    "from" TEXT NOT NULL,
    "timestamp" DATETIME NOT NULL,
    "payload" TEXT NOT NULL,
    "is_first_message" BOOLEAN NOT NULL
);
CREATE INDEX "target_idx" ON "messages"("target");
CREATE INDEX "stanza_id_idx" ON "messages"("stanza_id");
CREATE INDEX "timestamp_idx" ON "messages"("timestamp");