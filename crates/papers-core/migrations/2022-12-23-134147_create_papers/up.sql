-- Your SQL goes here
CREATE TABLE papers (
    id INTEGER NOT NULL PRIMARY KEY,
    url TEXT,
    filename TEXT,
    title TEXT,
    deleted BOOLEAN NOT NULL DEFAULT false,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
)
