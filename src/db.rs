use std::path::{Path, PathBuf};

use diesel::prelude::*;
use diesel::{Connection, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::{debug, warn};

mod models;
mod schema;

pub use models::*;

const DB_FILE_NAME: &str = "papers.db";

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct Db {
    connection: SqliteConnection,
}

fn db_filename(dir: &Path) -> PathBuf {
    dir.join(DB_FILE_NAME)
}

impl Db {
    pub fn init(dir: &Path) -> Self {
        let file = db_filename(dir);
        if file.is_file() {
            warn!(?file, "DB file already exists, can't init");
            panic!("Can't initialise, already a repo");
        }
        debug!(?file, "Initialising database");
        let connection = SqliteConnection::establish(&file.to_string_lossy()).unwrap();
        let mut s = Self { connection };
        s.migrate();
        s
    }

    pub fn load(dir: &Path) -> Self {
        let file = db_filename(dir);
        if !file.is_file() {
            warn!(?file, "DB file doesn't exist, not initialised yet");
            panic!("Not a repo, run `init` first");
        }
        debug!(?file, "Loading database");
        let connection = SqliteConnection::establish(&file.to_string_lossy()).unwrap();
        let mut s = Self { connection };
        s.migrate();
        s
    }

    pub fn migrate(&mut self) {
        self.connection.run_pending_migrations(MIGRATIONS).unwrap();
    }

    pub fn insert_paper(&mut self, paper: NewPaper) -> Paper {
        use schema::papers;

        diesel::insert_into(papers::table)
            .values(paper)
            .get_result(&mut self.connection)
            .expect("Failed to add paper")
    }

    pub fn insert_tags(&mut self, tags: Vec<NewTag>) {
        use schema::tags;
        for tag in tags {
            diesel::insert_into(tags::table)
                .values(tag)
                .execute(&mut self.connection)
                .expect("Failed to add tags");
        }
    }

    pub fn insert_labels(&mut self, labels: Vec<NewLabel>) {
        use schema::labels;
        for label in labels {
            diesel::insert_into(labels::table)
                .values(label)
                .execute(&mut self.connection)
                .expect("Failed to add labels");
        }
    }

    pub fn list_papers(&mut self) -> Vec<Paper> {
        use schema::papers::dsl::*;
        papers
            .load::<Paper>(&mut self.connection)
            .expect("Failed to load posts")
    }

    pub fn get_tags(&mut self, pid: i32) -> Vec<Tag> {
        use schema::tags::dsl::*;
        tags.filter(paper_id.eq(pid))
            .load::<Tag>(&mut self.connection)
            .unwrap_or_else(|_| panic!("Failed to get tags for paper id {}", pid))
    }

    pub fn get_labels(&mut self, pid: i32) -> Vec<Label> {
        use schema::labels::dsl::*;
        labels
            .filter(paper_id.eq(pid))
            .load::<Label>(&mut self.connection)
            .unwrap_or_else(|_| panic!("Failed to get labels for paper id {}", pid))
    }
}
