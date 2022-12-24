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
use tracing::{info, warn};

use papers::label::Label;

use crate::config::Config;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

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
    /// Remove a paper from being tracked.
    Remove {
        /// Id of the paper to remove.
        #[clap()]
        paper_id: i32,
    },
    /// Manage tags associated with a paper.
    Tags {
        #[clap(subcommand)]
        subcommand: TagsCommands,
    },
    /// Manage labels associated with a paper.
    Labels {
        #[clap(subcommand)]
        subcommand: LabelsCommands,
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
    /// Open the file for the given paper.
    Open {
        /// Id of the paper to open.
        #[clap()]
        paper_id: i32,
    },
}

impl SubCommand {
    pub fn execute(self, _config: &Config) {
        match self {
            Self::Init {} => {
                let cwd = current_dir().unwrap();
                let _ = Repo::init(&cwd);
                info!("Initialised the current directory")
            }
            Self::Fetch {
                url,
                name,
                title,
                tags,
                labels,
            } => {
                let client = reqwest::blocking::Client::builder()
                    .user_agent(APP_USER_AGENT)
                    .build()
                    .unwrap();
                let mut res = client.get(&url).send().expect("Failed to get url");
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
            Self::Add {
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
            Self::Update {
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
            Self::Remove { paper_id } => {
                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                repo.remove(paper_id);
                info!(id = paper_id, "Removed paper");
            }
            Self::Tags { subcommand } => {
                subcommand.execute();
            }
            Self::Labels { subcommand } => {
                subcommand.execute();
            }
            Self::List {
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
            Self::Notes { paper_id } => {
                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                let mut note = repo.get_note(paper_id);

                let mut file = tempfile::Builder::new()
                    .prefix(&format!("papers-{paper_id}-"))
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
            Self::Open { paper_id } => {
                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                let paper = repo.get_paper(paper_id);
                if let Some(paper) = paper {
                    info!(file = paper.filename, "Opening");
                    open::that(paper.filename).unwrap();
                } else {
                    warn!(id = paper_id, "No paper found");
                }
            }
        }
    }
}

fn edit(filename: &Path) {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_owned());
    Command::new(editor).arg(filename).status().unwrap();
}

#[derive(Debug, clap::Parser)]
pub enum TagsCommands {
    /// Add tags to a paper.
    Add {
        /// Id of the paper to add tags to.
        #[clap()]
        paper_id: i32,

        /// Tags to add.
        #[clap()]
        tags: Vec<Tag>,
    },
    /// Remove tags from a paper.
    Remove {
        /// Id of the paper to remove tags from.
        #[clap()]
        paper_id: i32,

        /// Tags to remove.
        #[clap()]
        tags: Vec<Tag>,
    },
}

impl TagsCommands {
    pub fn execute(self) {
        match self {
            Self::Add { paper_id, tags } => {
                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                repo.add_tags(paper_id, tags);
            }
            Self::Remove { paper_id, tags } => {
                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                repo.remove_tags(paper_id, tags);
            }
        }
    }
}

#[derive(Debug, clap::Parser)]
pub enum LabelsCommands {
    /// Add labels to a paper.
    Add {
        /// Id of the paper to add labels to.
        #[clap()]
        paper_id: i32,

        /// Labels to add.
        #[clap()]
        labels: Vec<Label>,
    },
    /// Remove labels from a paper.
    Remove {
        /// Id of the paper to remove labels from.
        #[clap()]
        paper_id: i32,

        /// Labels to remove.
        #[clap()]
        labels: Vec<Tag>,
    },
}

impl LabelsCommands {
    pub fn execute(self) {
        match self {
            Self::Add { paper_id, labels } => {
                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                repo.add_labels(paper_id, labels);
            }
            Self::Remove { paper_id, labels } => {
                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                repo.remove_labels(paper_id, labels);
            }
        }
    }
}
