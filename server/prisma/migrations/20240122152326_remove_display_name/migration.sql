/*
  Warnings:

  - You are about to drop the column `display_name` on the `User` table. All the data in the column will be lost.

*/
-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_User" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "password" TEXT NOT NULL,
    "profile_picture" TEXT,
    "online" BOOLEAN NOT NULL DEFAULT false
);
INSERT INTO "new_User" ("id", "online", "password", "profile_picture") SELECT "id", "online", "password", "profile_picture" FROM "User";
DROP TABLE "User";
ALTER TABLE "new_User" RENAME TO "User";
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
