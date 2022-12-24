-- Your SQL goes here
CREATE TABLE labels (
    paper_id INTEGER NOT NULL,
    label_key TEXT NOT NULL,
    label_value TEXT NOT NULL,
    FOREIGN KEY(paper_id) REFERENCES papers(id),
    PRIMARY KEY(paper_id, label_key)
)