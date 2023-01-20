use std::path::{Path, PathBuf};

use diesel::connection::SimpleConnection;
use diesel::sqlite::Sqlite;
use diesel::{debug_query, prelude::*};
use diesel::{Connection, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::{debug, warn};

mod models;
mod schema;

pub use models::*;

pub fn default_filename() -> PathBuf {
    "papers.db".into()
}

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct Db {
    connection: SqliteConnection,
}

impl Db {
    #[cfg(test)]
    pub fn in_memory() -> anyhow::Result<Self> {
        let connection = SqliteConnection::establish(":memory:")?;
        let mut s = Self { connection };
        s.migrate()?;
        Ok(s)
    }

    pub fn init(dir: &Path, file: &Path) -> anyhow::Result<Self> {
        let file = dir.join(file);
        if file.is_file() {
            warn!(?file, "DB file already exists, can't init");
            anyhow::bail!("Can't initialise, already a repo");
        }
        debug!(?file, "Initialising database");
        let connection = SqliteConnection::establish(&file.to_string_lossy())?;
        let mut s = Self { connection };
        s.migrate()?;
        debug!(?file, "Initialised database");
        Ok(s)
    }

    pub fn load(dir: &Path, file: &Path) -> anyhow::Result<Self> {
        let file = dir.join(file);
        if !file.is_file() {
            warn!(?file, "DB file doesn't exist, not initialised yet");
            anyhow::bail!("Not a repo, run `init` first");
        }
        debug!(?file, "Loading database");
        let connection = SqliteConnection::establish(&file.to_string_lossy())?;
        let mut s = Self { connection };
        s.migrate()?;
        Ok(s)
    }

    pub fn migrate(&mut self) -> anyhow::Result<()> {
        self.connection
            .batch_execute("PRAGMA foreign_keys = ON")
            .unwrap();
        self.connection.run_pending_migrations(MIGRATIONS).unwrap();
        Ok(())
    }

    pub fn insert_paper(&mut self, paper: NewPaper) -> anyhow::Result<Paper> {
        use schema::papers;
        let paper = diesel::insert_into(papers::table)
            .values(paper)
            .get_result(&mut self.connection)?;
        Ok(paper)
    }

    pub fn update_paper(&mut self, paper: PaperUpdate) -> anyhow::Result<()> {
        diesel::update(&paper)
            .set(&paper)
            .execute(&mut self.connection)?;
        Ok(())
    }

    pub fn remove_paper(&mut self, paper_id_to_remove: i32) -> anyhow::Result<()> {
        use schema::papers;
        use schema::papers::deleted;
        use schema::papers::id;
        let query = diesel::update(papers::table)
            .filter(id.eq(paper_id_to_remove))
            .set(deleted.eq(true));
        debug!(query=%debug_query::<Sqlite, _>(&query), "Removing paper");
        query.execute(&mut self.connection)?;
        Ok(())
    }

    pub fn insert_tags(&mut self, tags: Vec<NewTag>) -> anyhow::Result<()> {
        use schema::tags;
        use schema::tags::{paper_id, tag};
        for new_tag in tags {
            let query = diesel::insert_into(tags::table)
                .values(new_tag)
                .on_conflict((paper_id, tag))
                .do_nothing();
            debug!(query=%debug_query::<Sqlite, _>(&query), "Inserting tags");
            query.execute(&mut self.connection)?;
        }
        Ok(())
    }

    pub fn remove_tags(&mut self, tags_to_remove: Vec<NewTag>) -> anyhow::Result<()> {
        use schema::tags;
        use schema::tags::{paper_id, tag};
        for tag_to_remove in tags_to_remove {
            let query = diesel::delete(tags::table).filter(
                paper_id
                    .eq(tag_to_remove.paper_id)
                    .and(tag.eq(tag_to_remove.tag)),
            );
            debug!(query=%debug_query(&query), "Removing tags");
            query.execute(&mut self.connection)?;
        }
        Ok(())
    }

    pub fn insert_labels(&mut self, labels: Vec<NewLabel>) -> anyhow::Result<()> {
        use schema::labels;
        use schema::labels::{label_key, paper_id};
        for label in labels {
            let query = diesel::insert_into(labels::table)
                .values(label)
                .on_conflict((paper_id, label_key))
                .do_nothing();
            debug!(query=%debug_query::<Sqlite,_>(&query), "Inserting labels");
            query.execute(&mut self.connection)?;
        }
        Ok(())
    }

    pub fn remove_labels(&mut self, labels_to_remove: Vec<DeleteLabel>) -> anyhow::Result<()> {
        use schema::labels;
        use schema::labels::{label_key, paper_id};
        for label_to_remove in labels_to_remove {
            let query = diesel::delete(labels::table).filter(
                paper_id
                    .eq(label_to_remove.paper_id)
                    .and(label_key.eq(label_to_remove.label_key)),
            );
            debug!(query=%debug_query(&query), "Removing labels");
            query.execute(&mut self.connection)?;
        }
        Ok(())
    }

    pub fn get_paper(&mut self, paper_id: i32) -> anyhow::Result<Paper> {
        use schema::papers::dsl::papers;
        let res = papers.find(paper_id).first(&mut self.connection)?;
        Ok(res)
    }

    pub fn list_papers(&mut self) -> anyhow::Result<Vec<Paper>> {
        use schema::papers::dsl::papers;
        let res = papers.load::<Paper>(&mut self.connection)?;
        Ok(res)
    }

    pub fn insert_authors(&mut self, authors: Vec<NewAuthor>) -> anyhow::Result<()> {
        use schema::authors;
        use schema::authors::{author, paper_id};
        for new_author in authors {
            let query = diesel::insert_into(authors::table)
                .values(new_author)
                .on_conflict((paper_id, author))
                .do_nothing();
            debug!(query=%debug_query::<Sqlite, _>(&query), "Inserting authors");
            query.execute(&mut self.connection)?;
        }
        Ok(())
    }

    pub fn remove_authors(&mut self, authors_to_remove: Vec<NewAuthor>) -> anyhow::Result<()> {
        use schema::authors;
        use schema::authors::{author, paper_id};
        for author_to_remove in authors_to_remove {
            let query = diesel::delete(authors::table).filter(
                paper_id
                    .eq(author_to_remove.paper_id)
                    .and(author.eq(author_to_remove.author)),
            );
            debug!(query=%debug_query(&query), "Removing authors");
            query.execute(&mut self.connection)?;
        }
        Ok(())
    }

    pub fn get_authors(&mut self, pid: i32) -> anyhow::Result<Vec<Author>> {
        use schema::authors::dsl::{authors, paper_id};
        let res = authors
            .filter(paper_id.eq(pid))
            .load::<Author>(&mut self.connection)?;
        Ok(res)
    }

    pub fn get_tags(&mut self, pid: i32) -> anyhow::Result<Vec<Tag>> {
        use schema::tags::dsl::{paper_id, tags};
        let res = tags
            .filter(paper_id.eq(pid))
            .load::<Tag>(&mut self.connection)?;
        Ok(res)
    }

    pub fn get_labels(&mut self, pid: i32) -> anyhow::Result<Vec<Label>> {
        use schema::labels::dsl::{labels, paper_id};
        let res = labels
            .filter(paper_id.eq(pid))
            .load::<Label>(&mut self.connection)?;
        Ok(res)
    }

    pub fn get_note(&mut self, pid: i32) -> anyhow::Result<Option<Note>> {
        use schema::notes::dsl::{notes, paper_id};
        let res = notes
            .filter(paper_id.eq(pid))
            .first::<Note>(&mut self.connection)
            .optional()?;
        Ok(res)
    }

    pub fn insert_note(&mut self, note: NewNote) -> anyhow::Result<()> {
        use schema::notes;
        diesel::insert_into(notes::table)
            .values(note)
            .execute(&mut self.connection)?;
        Ok(())
    }

    pub fn update_note(&mut self, new_note: Note) -> anyhow::Result<()> {
        use schema::notes::dsl::{content, notes};
        diesel::update(notes.find(new_note.id))
            .set(content.eq(new_note.content))
            .execute(&mut self.connection)?;
        Ok(())
    }
}
