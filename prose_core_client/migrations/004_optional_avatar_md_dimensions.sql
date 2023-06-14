DROP TABLE "avatar_metadata";

CREATE TABLE "avatar_metadata" (
    "jid" TEXT PRIMARY KEY NOT NULL,
    "mime_type" TEXT NOT NULL,
    "checksum" TEXT NOT NULL,
    "width" INTEGER,
    "height" INTEGER,
    "updated_at" DATETIME NOT NULL
);