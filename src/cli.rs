use std::{
    env::current_dir,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use cli_table::{
    format::{Border, Separator},
    print_stdout, WithTitle,
};
use papers::{repo::Repo, tag::Tag};
use tracing::info;

use papers::label::Label;

use crate::config::Config;

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[clap(long, short)]
    pub config_file: Option<PathBuf>,

    #[clap(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum SubCommand {
    /// Initialise a new paper repository.
    Init {},
    // TODO: interactive fetch and add
    /// Fetch a paper pdf from a url and add it to the repo.
    Fetch {
        /// Url to fetch the pdf from.
        #[clap()]
        url: String,

        /// Name of the file to save it to. Defaults to the basename of the url.
        #[clap()]
        name: Option<String>,

        /// Title of the file.
        #[clap(long)]
        title: Option<String>,

        /// Tags to associate with this file.
        #[clap(name = "tag", long, short)]
        tags: Vec<Tag>,

        /// Labels to associate with this file. Labels take the form `key=value`.
        #[clap(name = "label", long, short)]
        labels: Vec<Label>,
    },
    /// Add a pdf from a local file to the repo.
    Add {
        /// File to add.
        #[clap()]
        file: PathBuf,

        /// Title of the file.
        #[clap(long)]
        title: Option<String>,

        /// Tags to associate with this file.
        #[clap(name = "tag", long, short)]
        tags: Vec<Tag>,

        /// Labels to associate with this file. Labels take the form `key=value`.
        #[clap(name = "label", long, short)]
        labels: Vec<Label>,
    },
    /// Update metadata about an existing paper.
    Update {
        /// Id of the paper.
        #[clap()]
        paper_id: i32,

        /// Url the paper was fetched from.
        #[clap(long, short)]
        url: Option<String>,

        /// File to add.
        #[clap(long, short)]
        file: Option<PathBuf>,

        /// Title of the file.
        #[clap(long)]
        title: Option<String>,
    },
    /// List the papers stored with this repo.
    List {
        /// Filter down to papers whose titles match this (case-insensitive).
        #[clap(long)]
        title: Option<String>,

        /// Filter down to papers that have all of the given tags.
        #[clap(name = "tag", long, short)]
        tags: Vec<Tag>,

        /// Filter down to papers that have all of the given labels. Labels take the form `key=value`.
        #[clap(name = "label", long, short)]
        labels: Vec<Label>,
    },
    /// Manage notes associated with a paper.
    Notes {
        #[clap()]
        paper_id: i32,
        // TODO: create another nested subcommand for show, edit, ..
    },
}

impl SubCommand {
    pub fn execute(self, _config: &Config) {
        match self {
            SubCommand::Init {} => {
                let cwd = current_dir().unwrap();
                Repo::init(&cwd);
                info!("Initialised the current directory")
            }
            SubCommand::Fetch {
                url,
                name,
                title,
                tags,
                labels,
            } => {
                let mut res = reqwest::blocking::get(&url).expect("Failed to get url");
                let filename = if let Some(name) = name {
                    name
                } else {
                    url.split('/').last().as_ref().unwrap().to_string()
                };
                let mut file = File::create(&filename).unwrap();
                std::io::copy(&mut res, &mut file).unwrap();

                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                let paper = repo.add(&filename, Some(url), title, tags, labels);
                info!(id = paper.id, filename = paper.filename, "Added paper");
            }
            SubCommand::Add {
                file,
                title,
                tags,
                labels,
            } => {
                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                let paper = repo.add(&file, None, title, tags, labels);
                info!(id = paper.id, filename = paper.filename, "Added paper");
            }
            SubCommand::Update {
                paper_id,
                url,
                file,
                title,
            } => {
                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                let url = if let Some(s) = url {
                    if s.is_empty() {
                        Some(None)
                    } else {
                        Some(Some(s))
                    }
                } else {
                    None
                };
                let title = if let Some(s) = title {
                    if s.is_empty() {
                        Some(None)
                    } else {
                        Some(Some(s))
                    }
                } else {
                    None
                };
                repo.update(paper_id, file.as_ref(), url, title);
                info!(id = paper_id, "Updated paper");
            }
            SubCommand::List {
                title,
                tags,
                labels,
            } => {
                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                let papers = repo.list(title, tags, labels);

                let table = papers
                    .with_title()
                    .border(Border::builder().build())
                    .separator(Separator::builder().build());
                print_stdout(table).unwrap();
            }
            SubCommand::Notes { paper_id } => {
                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                let mut note = repo.get_note(paper_id);

                let mut file = tempfile::Builder::new()
                    .prefix(&format!("papers-{}-", paper_id))
                    .suffix(".md")
                    .rand_bytes(5)
                    .tempfile()
                    .unwrap();
                write!(file, "{}", note.content).unwrap();

                edit(file.path());

                let mut content = String::new();
                let mut file = File::open(file.path()).unwrap();
                file.read_to_string(&mut content).unwrap();
                note.content = content;
                repo.update_note(note);
            }
        }
    }
}

fn edit(filename: &Path) {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_owned());
    Command::new(editor).arg(filename).status().unwrap();
}
