-- Your SQL goes here
CREATE TABLE tags (
    paper_id INTEGER NOT NULL,
    tag TEXT NOT NULL,
    FOREIGN KEY(paper_id) REFERENCES papers(id),
    PRIMARY KEY(paper_id, tag)
)