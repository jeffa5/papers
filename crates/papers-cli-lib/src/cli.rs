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
use gray_matter::{engine::YAML, Matter};
use papers_core::{author::Author, repo::Repo, tag::Tag};
use tracing::{debug, info, warn};

use papers_core::label::Label;

use crate::{config::Config, url_path::UrlOrPath};
use crate::{ids::Ids, table::TablePaper};

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

fn now_naive() -> chrono::NaiveDateTime {
    chrono::Utc::now().naive_utc()
}

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
    /// Add a paper document from a url or local file and add it to the repo.
    Add {
        /// Url to fetch from or path of a local file in the repo.
        #[clap()]
        url_or_path: UrlOrPath,

        /// Name of the file to save it to. Defaults to the basename of the url.
        #[clap(long)]
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
    /// Update metadata about an existing paper.
    Update {
        /// Ids of papers to update, e.g. 1 1,2 1-3,5.
        #[clap()]
        ids: Ids,

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
        /// Ids of papers to remove, e.g. 1 1,2 1-3,5.
        #[clap()]
        ids: Ids,

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
        /// Paper ids to filter to, e.g. 1 1,2 1-3,5.
        #[clap(default_value_t)]
        ids: Ids,
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
        /// Ids of papers to show information for, e.g. 1 1,2 1-3,5.
        #[clap()]
        ids: Ids,

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
            Self::Add {
                url_or_path,
                name,
                title,
                authors,
                tags,
                labels,
            } => match url_or_path {
                UrlOrPath::Url(url) => {
                    debug!(user_agent = APP_USER_AGENT, "Building http client");
                    let filename = if let Some(name) = name {
                        name
                    } else {
                        url.path_segments().unwrap().last().unwrap().to_owned()
                    };

                    if PathBuf::from(&filename).exists() {
                        anyhow::bail!("Path {} already exists, try setting a name", filename);
                    }

                    let client = reqwest::blocking::Client::builder()
                        .user_agent(APP_USER_AGENT)
                        .build()?;
                    info!(%url, "Fetching");
                    let mut res = client
                        .get(url.clone())
                        .send()
                        .expect("Failed to get url")
                        .error_for_status()?;
                    let headers = res.headers();
                    if let Some(content_type) = headers.get(http::header::CONTENT_TYPE) {
                        if content_type != "application/pdf" {
                            warn!("File fetched was not a pdf, perhaps it needs authorisation?")
                        }
                    }

                    let mut file = File::create(&filename)?;
                    debug!(%url, filename, "Saving");
                    std::io::copy(&mut res, &mut file)?;
                    info!(%url, filename, "Fetched");
                    add(
                        &filename,
                        Some(url.to_string()),
                        title,
                        authors,
                        tags,
                        labels,
                    )?;
                }
                UrlOrPath::Path(path) => {
                    add(path, None, title, authors, tags, labels)?;
                }
            },
            Self::Update {
                ids,
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
                for id in ids.0 {
                    repo.update(id, file.as_ref(), url.clone(), title.clone())?;
                    info!(id, "Updated paper");
                }
            }
            Self::Remove { ids, with_file } => {
                let mut repo = load_repo()?;
                for id in ids.0 {
                    if let Ok(paper) = repo.get_paper(id) {
                        debug!(id, file = paper.filename, "Removing paper");
                        repo.remove(id)?;
                        info!(id, file = paper.filename, "Removed paper");
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
                        info!(id, "No paper with that id to remove");
                    }
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
                let papers = repo.list(ids.0, file, title, authors, tags, labels)?;

                match output {
                    OutputStyle::Table => {
                        let now = now_naive();
                        let table_papers = papers
                            .into_iter()
                            .map(|paper| TablePaper::from_paper(paper, now))
                            .collect::<Vec<_>>();
                        let table = table_papers
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
            Self::Show { ids, output } => {
                let mut repo = load_repo()?;
                let mut papers = Vec::new();
                for id in ids.0 {
                    let paper = repo.get_paper(id)?;
                    papers.push(paper);
                }
                match output {
                    OutputStyle::Table => {
                        let now = now_naive();
                        let table_papers = papers
                            .into_iter()
                            .map(|paper| TablePaper::from_paper(paper, now))
                            .collect::<Vec<_>>();
                        let table = table_papers
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
            Self::Notes { paper_id } => {
                let mut repo = load_repo()?;
                let mut note = repo.get_note(paper_id)?;

                let paper = repo.get_paper(paper_id)?;
                let dont_edit = "# Do not edit this metadata, write notes below.";
                let content = format!(
                    "---\n{}{dont_edit}\n---\n\n{}",
                    serde_yaml::to_string(&paper).unwrap(),
                    note.content
                );

                let mut file = tempfile::Builder::new()
                    .prefix(&format!("papers-{paper_id}-"))
                    .suffix(".md")
                    .rand_bytes(5)
                    .tempfile()?;
                write!(file, "{}", content)?;

                edit(file.path())?;

                let mut content = String::new();
                let mut file = File::open(file.path())?;
                file.read_to_string(&mut content)?;

                let matter = Matter::<YAML>::new();
                let result = matter.parse(&content);

                note.content = result.content;
                repo.update_note(note)?;
            }
            Self::Open { paper_id } => {
                let mut repo = load_repo()?;
                let paper = repo.get_paper(paper_id)?;
                info!(file = paper.filename, "Opening");
                open::that(paper.filename)?;
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
