-- This file should undo anything in `up.sql`
ALTER TABLE papers
DROP COLUMN created_at;

ALTER TABLE papers
DROP COLUMN modified_at;