DROP TABLE "roster_item";

CREATE TABLE "roster_item" (
    "jid" TEXT PRIMARY KEY NOT NULL,
    "name" TEXT,
    "subscription" TEXT NOT NULL,
    "groups" TEXT
);