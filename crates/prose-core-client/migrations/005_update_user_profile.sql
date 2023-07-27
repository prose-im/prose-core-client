DROP TABLE "user_profile";

CREATE TABLE "user_profile" (
    "jid" TEXT PRIMARY KEY NOT NULL,
    "first_name" TEXT,
    "last_name" TEXT,
    "nickname" TEXT,
    "org" TEXT,
    "role" TEXT,
    "title" TEXT,
    "email" TEXT,
    "tel" TEXT,
    "url" TEXT,
    "locality" TEXT,
    "country" TEXT,
    "updated_at" DATETIME NOT NULL
);