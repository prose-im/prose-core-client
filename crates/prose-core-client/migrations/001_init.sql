CREATE TABLE "kv" (
    "key" TEXT PRIMARY KEY NOT NULL,
    "value" BLOB NOT NULL
);

CREATE TABLE "roster_item" (
    "jid" TEXT PRIMARY KEY NOT NULL,
    "subscription" TEXT NOT NULL,
    "groups" TEXT
);

CREATE TABLE "user_profile" (
    "jid" TEXT PRIMARY KEY NOT NULL,
    "full_name" TEXT,
    "nickname" TEXT,
    "org" TEXT,
    "title" TEXT,
    "email" TEXT,
    "tel" TEXT,
    "url" TEXT,
    "locality" TEXT,
    "country" TEXT,
    "updated_at" DATETIME NOT NULL
);

CREATE TABLE "avatar_metadata" (
    "jid" TEXT PRIMARY KEY NOT NULL,
    "mime_type" TEXT NOT NULL,
    "checksum" TEXT NOT NULL,
    "width" INTEGER NOT NULL,
    "height" INTEGER NOT NULL,
    "updated_at" DATETIME NOT NULL
);