-- Your SQL goes here
CREATE TABLE notes (
    id INTEGER NOT NULL PRIMARY KEY,
    paper_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    FOREIGN KEY(paper_id) REFERENCES papers(id)
)