use std::path::{Path, PathBuf};

use diesel::prelude::*;
use diesel::{Connection, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::{debug, warn};

use crate::models::{NewPaper, Paper};

const DB_FILE_NAME: &str = "test.db";

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

    pub fn insert_paper(&mut self, paper: NewPaper) {
        use crate::schema::papers;
        diesel::insert_into(papers::table)
            .values(paper)
            .execute(&mut self.connection)
            .expect("Failed to add paper");
    }

    pub fn list(&mut self) -> Vec<Paper> {
        use crate::schema::papers::dsl::*;
        papers
            .load::<Paper>(&mut self.connection)
            .expect("Failed to load posts")
    }
}
