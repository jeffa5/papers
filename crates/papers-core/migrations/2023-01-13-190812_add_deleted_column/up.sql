-- Your SQL goes here
ALTER TABLE papers
ADD COLUMN deleted BOOLEAN NOT NULL DEFAULT false;