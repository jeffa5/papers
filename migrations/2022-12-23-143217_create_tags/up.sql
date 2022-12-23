-- Your SQL goes here
CREATE TABLE tags (
    id INTEGER NOT NULL PRIMARY KEY,
    paper_id INTEGER NOT NULL,
    tag TEXT NOT NULL,
    FOREIGN KEY(paper_id) REFERENCES papers(id)
)