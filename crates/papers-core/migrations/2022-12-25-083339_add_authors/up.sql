-- Your SQL goes here

CREATE TABLE authors (
    paper_id INTEGER NOT NULL,
    author TEXT NOT NULL,
    FOREIGN KEY(paper_id) REFERENCES papers(id),
    PRIMARY KEY(paper_id, author)
)