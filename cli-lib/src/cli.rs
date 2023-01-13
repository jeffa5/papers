use std::{
    env::current_dir,
    fs::{remove_file, File},
    io::{stdout, Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use clap::{CommandFactory, ValueEnum};
use clap_complete::{generate_to, Generator, Shell};
use cli_table::{
    format::{Border, Separator},
    print_stdout, WithTitle,
};
use papers_core::{author::Author, repo::Repo, tag::Tag};
use tracing::{debug, info, warn};

use papers_core::label::Label;

use crate::config::Config;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// A paper management program.
#[derive(Debug, clap::Parser)]
pub struct Cli {
    /// Config file path to load.
    #[clap(long, short)]
    pub config_file: Option<PathBuf>,

    /// Commands.
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

/// Subcommands for the cli.
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

        /// Authors to associate with this file.
        #[clap(name = "author", long, short)]
        authors: Vec<Author>,

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

        /// Authors to associate with this file.
        #[clap(name = "author", long, short)]
        authors: Vec<Author>,

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

        /// Also remove the paper file.
        #[clap(long)]
        with_file: bool,
    },
    /// Manage authors associated with a paper.
    Authors {
        /// Subcommands for managing authors.
        #[clap(subcommand)]
        subcommand: AuthorsCommands,
    },
    /// Manage tags associated with a paper.
    Tags {
        /// Subcommands for managing tags.
        #[clap(subcommand)]
        subcommand: TagsCommands,
    },
    /// Manage labels associated with a paper.
    Labels {
        /// Subcommands for managing labels.
        #[clap(subcommand)]
        subcommand: LabelsCommands,
    },
    /// List the papers stored with this repo.
    List {
        /// Paper id's to filter to.
        #[clap(name = "id", long)]
        ids: Vec<i32>,

        /// Filter down to papers that have filenames which match this (case-insensitive).
        #[clap(long, short)]
        file: Option<String>,

        /// Filter down to papers whose titles match this (case-insensitive).
        #[clap(long)]
        title: Option<String>,

        /// Filter down to papers that have all of the given authors.
        #[clap(name = "author", long, short)]
        authors: Vec<Author>,

        /// Filter down to papers that have all of the given tags.
        #[clap(name = "tag", long, short)]
        tags: Vec<Tag>,

        /// Filter down to papers that have all of the given labels. Labels take the form `key=value`.
        #[clap(name = "label", long, short)]
        labels: Vec<Label>,

        /// Output the filtered selection of papers in different formats.
        #[clap(long, short, value_enum, default_value_t)]
        output: OutputStyle,
    },
    /// Show all information about a paper.
    Show {
        /// Id of the paper to show information for.
        #[clap()]
        paper_id: i32,

        /// Output the paper in different formats.
        #[clap(long, short, value_enum, default_value_t)]
        output: OutputStyle,
    },
    /// Manage notes associated with a paper.
    Notes {
        /// Id of the paper to update notes for.
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
    /// Generate cli completion files.
    Completions {
        /// Shell to generate for.
        #[clap()]
        shell: Shell,
        /// Directory to save completion files to.
        #[clap(default_value = ".")]
        dir: PathBuf,
    },
}

impl SubCommand {
    /// Execute a subcommand.
    pub fn execute(self, _config: &Config) -> anyhow::Result<()> {
        match self {
            Self::Init {} => {
                let cwd = current_dir()?;
                let _ = Repo::init(&cwd);
                info!("Initialised the current directory");
            }
            Self::Fetch {
                url,
                name,
                title,
                authors,
                tags,
                labels,
            } => {
                debug!(user_agent = APP_USER_AGENT, "Building http client");
                let client = reqwest::blocking::Client::builder()
                    .user_agent(APP_USER_AGENT)
                    .build()?;
                info!(url, "Fetching");
                let mut res = client
                    .get(&url)
                    .send()
                    .expect("Failed to get url")
                    .error_for_status()?;
                let headers = res.headers();
                if let Some(content_type) = headers.get(http::header::CONTENT_TYPE) {
                    if content_type != "application/pdf" {
                        warn!("File fetched was not a pdf, perhaps it needs authorisation?")
                    }
                }

                let filename = if let Some(name) = name {
                    name
                } else {
                    url.split('/')
                        .last()
                        .as_ref()
                        .unwrap_or(&url.as_str())
                        .to_string()
                };
                let mut file = File::create(&filename)?;
                debug!(url, filename, "Saving");
                std::io::copy(&mut res, &mut file)?;
                info!(url, filename, "Fetched");

                add(&filename, Some(url), title, authors, tags, labels)?;
            }
            Self::Add {
                file,
                title,
                authors,
                tags,
                labels,
            } => {
                add(file, None, title, authors, tags, labels)?;
            }
            Self::Update {
                paper_id,
                url,
                file,
                title,
            } => {
                let mut repo = load_repo()?;
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
                repo.update(paper_id, file.as_ref(), url, title)?;
                info!(id = paper_id, "Updated paper");
            }
            Self::Remove {
                paper_id,
                with_file,
            } => {
                let mut repo = load_repo()?;
                if let Ok(Some(paper)) = repo.get_paper(paper_id) {
                    debug!(id = paper_id, file = paper.filename, "Removing paper");
                    repo.remove(paper_id)?;
                    info!(id = paper_id, file = paper.filename, "Removed paper");
                    if with_file {
                        // check that the file isn't needed by another paper
                        let papers_with_that_file = repo.list(
                            Vec::new(),
                            Some(paper.filename.clone()),
                            None,
                            Vec::new(),
                            Vec::new(),
                            Vec::new(),
                        )?;
                        if papers_with_that_file.is_empty() {
                            debug!(file = paper.filename, "Removing file");
                            remove_file(&paper.filename)?;
                            info!(file = paper.filename, "Removed file");
                        } else {
                            let papers_with_that_file = papers_with_that_file
                                .iter()
                                .map(|p| p.id)
                                .collect::<Vec<_>>();
                            warn!(
                                file = paper.filename,
                                ?papers_with_that_file,
                                "Can't remove the file, it is used by other papers"
                            );
                        }
                    }
                } else {
                    info!(id = paper_id, "No paper with that id to remove");
                }
            }
            Self::Authors { subcommand } => {
                subcommand.execute()?;
            }
            Self::Tags { subcommand } => {
                subcommand.execute()?;
            }
            Self::Labels { subcommand } => {
                subcommand.execute()?;
            }
            Self::List {
                ids,
                file,
                title,
                authors,
                tags,
                labels,
                output,
            } => {
                let mut repo = load_repo()?;
                let papers = repo.list(ids, file, title, authors, tags, labels)?;

                match output {
                    OutputStyle::Table => {
                        let table = papers
                            .with_title()
                            .border(Border::builder().build())
                            .separator(Separator::builder().build());
                        print_stdout(table)?;
                    }
                    OutputStyle::Json => {
                        serde_json::to_writer(stdout(), &papers)?;
                    }
                    OutputStyle::Yaml => {
                        serde_yaml::to_writer(stdout(), &papers)?;
                    }
                }
            }
            Self::Show { paper_id, output } => {
                let mut repo = load_repo()?;
                let paper = repo.get_paper(paper_id)?;
                if let Some(paper) = paper {
                    match output {
                        OutputStyle::Table => {
                            let table = vec![paper]
                                .with_title()
                                .border(Border::builder().build())
                                .separator(Separator::builder().build());
                            print_stdout(table)?;
                        }
                        OutputStyle::Json => {
                            serde_json::to_writer(stdout(), &paper)?;
                        }
                        OutputStyle::Yaml => {
                            serde_yaml::to_writer(stdout(), &paper)?;
                        }
                    }
                }
            }
            Self::Notes { paper_id } => {
                let mut repo = load_repo()?;
                let mut note = repo.get_note(paper_id)?;

                let mut file = tempfile::Builder::new()
                    .prefix(&format!("papers-{paper_id}-"))
                    .suffix(".md")
                    .rand_bytes(5)
                    .tempfile()?;
                write!(file, "{}", note.content)?;

                edit(file.path())?;

                let mut content = String::new();
                let mut file = File::open(file.path())?;
                file.read_to_string(&mut content)?;
                note.content = content;
                repo.update_note(note)?;
            }
            Self::Open { paper_id } => {
                let mut repo = load_repo()?;
                let paper = repo.get_paper(paper_id)?;
                if let Some(paper) = paper {
                    info!(file = paper.filename, "Opening");
                    open::that(paper.filename)?;
                } else {
                    warn!(id = paper_id, "No paper found");
                }
            }
            Self::Completions { shell, dir } => {
                let path = gen_completions(shell, &dir);
                info!(?path, ?shell, "Generated completions");
            }
        }
        Ok(())
    }
}

fn load_repo() -> anyhow::Result<Repo> {
    let cwd = current_dir()?;
    let repo = Repo::load(&cwd)?;
    Ok(repo)
}

fn edit(filename: &Path) -> anyhow::Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_owned());
    Command::new(editor).arg(filename).status()?;
    Ok(())
}

/// Manage authors.
#[derive(Debug, clap::Parser)]
pub enum AuthorsCommands {
    /// Add authors to a paper.
    Add {
        /// Id of the paper to add authors to.
        #[clap()]
        paper_id: i32,

        /// Authors to add.
        #[clap()]
        authors: Vec<Author>,
    },
    /// Remove authors from a paper.
    Remove {
        /// Id of the paper to remove authors from.
        #[clap()]
        paper_id: i32,

        /// Authors to remove.
        #[clap()]
        authors: Vec<Author>,
    },
}

impl AuthorsCommands {
    /// Execute authors commands.
    pub fn execute(self) -> anyhow::Result<()> {
        match self {
            Self::Add { paper_id, authors } => {
                let mut repo = load_repo()?;
                repo.add_authors(paper_id, authors)?;
            }
            Self::Remove { paper_id, authors } => {
                let mut repo = load_repo()?;
                repo.remove_authors(paper_id, authors)?;
            }
        }
        Ok(())
    }
}

/// Manage tags.
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
    /// Execute tags commands.
    pub fn execute(self) -> anyhow::Result<()> {
        match self {
            Self::Add { paper_id, tags } => {
                let mut repo = load_repo()?;
                repo.add_tags(paper_id, tags)?;
            }
            Self::Remove { paper_id, tags } => {
                let mut repo = load_repo()?;
                repo.remove_tags(paper_id, tags)?;
            }
        }
        Ok(())
    }
}

/// Manage labels.
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
    /// Execute label commands.
    pub fn execute(self) -> anyhow::Result<()> {
        match self {
            Self::Add { paper_id, labels } => {
                let mut repo = load_repo()?;
                repo.add_labels(paper_id, labels)?;
            }
            Self::Remove { paper_id, labels } => {
                let mut repo = load_repo()?;
                repo.remove_labels(paper_id, labels)?;
            }
        }
        Ok(())
    }
}

fn add<P: AsRef<Path>>(
    file: P,
    url: Option<String>,
    mut title: Option<String>,
    mut authors: Vec<Author>,
    tags: Vec<Tag>,
    labels: Vec<Label>,
) -> anyhow::Result<()> {
    let file = file.as_ref();
    let mut repo = load_repo()?;

    if title.is_none() {
        title = extract_title(file);
    }

    if authors.is_empty() {
        authors = extract_authors(file);
    }

    let paper = repo.add(&file, url, title, authors, tags, labels)?;
    info!(id = paper.id, filename = paper.filename, "Added paper");

    Ok(())
}

fn extract_title(file: &Path) -> Option<String> {
    if let Ok(pdf_file) = pdf::file::File::<Vec<u8>>::open(file) {
        debug!(?file, "Loaded pdf file");
        if let Some(info) = pdf_file.trailer.info_dict.as_ref() {
            debug!(?file, ?info, "Found the info dict");
            // try and extract the title
            if let Some(found_title) = info.get("Title") {
                debug!(?file, "Found title");
                if let Ok(found_title) = found_title
                    .as_string()
                    .map(|ft| ft.as_str().unwrap_or_default().into_owned())
                {
                    if !found_title.is_empty() {
                        debug!(?file, title = found_title, "Setting auto title");
                        return Some(found_title);
                    }
                }
            }
        }
    }
    warn!("Couldn't find a title in pdf metadata");
    None
}

fn extract_authors(file: &Path) -> Vec<Author> {
    if let Ok(pdf_file) = pdf::file::File::<Vec<u8>>::open(file) {
        debug!(?file, "Loaded pdf file");
        if let Some(info) = pdf_file.trailer.info_dict.as_ref() {
            debug!(?file, ?info, "Found the info dict");
            // try and extract the authors
            if let Some(found_authors) = info.get("Author") {
                debug!(?file, "Found authors");
                if let Ok(found_authors) = found_authors
                    .as_string()
                    .map(|ft| ft.as_str().unwrap_or_default().into_owned())
                {
                    if !found_authors.is_empty() {
                        debug!(?file, authors = found_authors, "Setting auto authors");
                        return found_authors
                            .split(|c: char| !c.is_alphanumeric() && !c.is_whitespace())
                            .map(|a| a.trim())
                            .map(Author::new)
                            .collect();
                    }
                }
            }
        }
    }
    warn!("Couldn't find authors in pdf metadata");
    Vec::new()
}

/// Output style for lists.
#[derive(Debug, Default, Clone, ValueEnum)]
pub enum OutputStyle {
    /// Pretty table format.
    #[default]
    Table,
    /// Json format.
    Json,
    /// Yaml format.
    Yaml,
}

/// Generate completions.
pub fn gen_completions<S>(shell: S, outdir: &Path) -> anyhow::Result<PathBuf>
where
    S: Generator,
{
    let mut cmd = Cli::command();

    let path = generate_to(
        shell, &mut cmd, // We need to specify what generator to use
        "papers", // We need to specify the bin name manually
        outdir,   // We need to specify where to write to
    )?;
    Ok(path)
}

#[test]
fn verify_command() {
    Cli::command().debug_assert();
}
