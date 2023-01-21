use std::{
    collections::BTreeSet,
    env::current_dir,
    fs::{remove_file, rename, File},
    io::{stdin, stdout, Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use clap::{CommandFactory, ValueEnum};
use clap_complete::{generate_to, Generator, Shell};
use gray_matter::{engine::YAML, Matter};
use papers_core::{
    author::Author,
    paper::Paper,
    repo::{self, Repo},
    tag::Tag,
};
use reqwest::Url;
use tracing::{debug, info, warn};

use papers_core::label::Label;

use crate::{
    config::{Config, PathOrString},
    interactive::{input_bool, input_default, input_opt, input_vec, input_vec_default},
    table::Table,
};
use crate::{error, rename_files};
use crate::{file_or_stdin::FileOrStdin, ids::Ids};

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// A paper management program.
#[derive(Debug, clap::Parser)]
pub struct Cli {
    /// Config file path to load.
    #[clap(long, short, global = true)]
    pub config_file: Option<PathBuf>,

    /// Default repo to use if not found in parents of current directory.
    #[clap(long, global = true)]
    pub default_repo: Option<PathBuf>,

    /// Filename for the database.
    #[clap(long, global = true)]
    pub db_filename: Option<PathBuf>,

    /// Commands.
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

/// Subcommands for the cli.
#[derive(Debug, clap::Subcommand)]
pub enum SubCommand {
    /// Initialise a new paper repository.
    Init {
        /// Directory to initialise.
        #[clap(default_value = ".")]
        dir: PathBuf,
    },
    /// Add a paper to the repo.
    Add {
        /// Url to fetch from.
        #[clap(long, short)]
        url: Option<Url>,

        /// Whether to fetch the document from URL or not.
        #[clap(long)]
        fetch: Option<bool>,

        /// File to add.
        #[clap(long, short)]
        file: Option<PathBuf>,

        /// Title of the file.
        #[clap(long)]
        title: Option<String>,

        /// Authors to associate with these files.
        #[clap(name = "author", long, short)]
        authors: Vec<Author>,

        /// Tags to associate with these files.
        #[clap(name = "tag", long, short)]
        tags: Vec<Tag>,

        /// Labels to associate with these files. Labels take the form `key=value`.
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
    /// Remove papers from being tracked.
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
        #[clap()]
        ids: Option<Ids>,

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

        /// Show papers that have been deleted too.
        #[clap(long)]
        deleted: bool,

        /// Output the filtered selection of papers in different formats.
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
    /// Automatically rename files to match their entry in the database.
    RenameFiles {
        /// Strategy to use in renaming.
        #[clap(required = true)]
        strategies: Vec<rename_files::Strategy>,

        /// Print information but don't perform renaming.
        #[clap(long)]
        dry_run: bool,
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
    /// Import a list of tasks in json format.
    ///
    /// The format can be exported from a `list` command using the `-o json` argument.
    Import {
        /// File to import from, or '-' for stdin.
        #[clap()]
        file: FileOrStdin,
    },
}

impl SubCommand {
    /// Execute a subcommand.
    pub fn execute(self, config: &Config) -> anyhow::Result<()> {
        match self {
            Self::Init { dir } => {
                Repo::init(&dir, &config.db_filename)?;
                info!(?dir, "Initialised the directory");
            }
            Self::Add {
                mut url,
                mut fetch,
                mut file,
                mut title,
                mut authors,
                mut tags,
                mut labels,
            } => {
                let mut repo = load_repo(config)?;
                if atty::is(atty::Stream::Stdout) {
                    if let Some(url) = &url {
                        println!("Using url {}", url);
                    } else {
                        url = input_opt::<Url>("Url for document");
                    }

                    if let Some(fetch) = fetch {
                        if fetch {
                            println!("Will fetch url");
                        } else {
                            println!("Will not fetch url");
                        }
                    } else {
                        if let Some(url) = &url {
                            fetch = Some(input_bool(&format!("Fetch {}", url), false));
                        }
                    }

                    if let Some(file) = &file {
                        println!("Using file {:?}", file);
                    } else {
                        if let Some((url, true)) = url.as_ref().zip(fetch) {
                            // try and get the default filename to use
                            let default_file =
                                url.path_segments().unwrap().last().unwrap().to_owned();
                            file = Some(input_default::<PathBuf>("Path to file", &default_file));
                        } else {
                            file = input_opt::<PathBuf>("Path to file");
                        };
                    }

                    if let Some(true) = fetch {
                        if let Some(url) = &url {
                            if let Some(f) = &file {
                                let name = f.file_name().unwrap();
                                let path = repo.root().join(name);
                                file = Some(fetch_url(&url, &path)?);
                            }else {
                                anyhow::bail!("No file to downlod to");
                            }
                        }
                    }

                    if let Some(title) = &title {
                        println!("Using title {}", title);
                    } else {
                        let extracted_title = if let Some(file) = &file {
                            extract_title(file)
                        } else {
                            None
                        };
                        if let Some(extracted_title) = extracted_title {
                            title = Some(input_default("Title", &extracted_title));
                        } else {
                            title = input_opt("Title");
                        }
                    }

                    if authors.is_empty() {
                        let extracted_authors = if let Some(file) = &file {
                            extract_authors(file)
                        } else {
                            BTreeSet::new()
                        };
                        if extracted_authors.is_empty() {
                            authors = input_vec("Authors", ",");
                        } else {
                            let extracted_authors_str = extracted_authors
                                .iter()
                                .map(|a| a.to_string())
                                .collect::<Vec<String>>()
                                .join(",");
                            authors = input_vec_default("Authors", &extracted_authors_str, ",");
                        }
                    } else {
                        let authors_string = authors
                            .iter()
                            .map(|a| a.to_string())
                            .collect::<Vec<String>>()
                            .join(",");
                        println!("Using authors {}", authors_string);
                    }

                    let default_tags = &config.paper_defaults.tags;
                    if tags.is_empty() {
                        let default_tags_str = default_tags
                            .iter()
                            .map(|t| t.to_string())
                            .collect::<Vec<String>>()
                            .join(",");
                        tags = input_vec(&format!("Tags (default: {})", default_tags_str), " ");
                    } else {
                        let tags_string = tags
                            .iter()
                            .map(|t| t.to_string())
                            .collect::<Vec<String>>()
                            .join(",");
                        println!("Using tags {}", tags_string);
                    }
                    tags.extend(default_tags.iter().cloned());

                    let default_labels = &config.paper_defaults.labels;
                    if labels.is_empty() {
                        let default_labels_str = default_labels
                            .iter()
                            .map(|l| l.to_string())
                            .collect::<Vec<String>>()
                            .join(",");
                        labels = input_vec(
                            &format!("Labels (key=value) (default: {})", default_labels_str),
                            " ",
                        );
                    } else {
                        let labels_string = labels
                            .iter()
                            .map(|l| l.to_string())
                            .collect::<Vec<String>>()
                            .join(",");
                        println!("Using labels {}", labels_string);
                    }
                    labels.extend(default_labels.iter().cloned());
                } else {
                    if let Some(true) = fetch {
                        if let Some(url) = &url {
                            file = Some(fetch_url(&url, &file.unwrap())?);
                        }
                    }

                    if let Some(file) = &file {
                        if title.is_none() {
                            title = extract_title(file);
                        }

                        if authors.is_empty() {
                            authors = Vec::from_iter(extract_authors(file));
                        }
                    }
                }

                let authors = BTreeSet::from_iter(authors);
                let tags = BTreeSet::from_iter(tags);
                let labels = BTreeSet::from_iter(labels);

                let url = url.map(|u| u.to_string());

                match add(
                    &mut repo,
                    file,
                    url,
                    title.clone(),
                    authors.clone(),
                    tags.clone(),
                    labels.clone(),
                ) {
                    Ok(paper) => {
                        println!("Added paper {}", paper.id);
                    }
                    Err(err) => {
                        warn!(%err, "Failed to add paper");
                        error!("Failed to add paper: {}", err);
                    }
                }
            }
            Self::Update {
                ids,
                url,
                file,
                title,
            } => {
                let mut repo = load_repo(config)?;
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
                let mut repo = load_repo(config)?;
                for id in ids.0 {
                    if let Ok(paper) = repo.get_paper(id) {
                        debug!(id, file = paper.filename, "Removing paper");
                        repo.remove(id)?;
                        info!(id, file = paper.filename, "Removed paper");
                        if with_file {
                            // check that the file isn't needed by another paper
                            let papers_with_that_file = repo.list(
                                Vec::new(),
                                paper.filename.clone(),
                                None,
                                Vec::new(),
                                Vec::new(),
                                Vec::new(),
                                false,
                            )?;
                            if papers_with_that_file.is_empty() {
                                if let Some(filename) = &paper.filename {
                                    debug!(file = filename, "Removing file");
                                    remove_file(filename)?;
                                    info!(file = filename, "Removed file");
                                }
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
                subcommand.execute(config)?;
            }
            Self::Tags { subcommand } => {
                subcommand.execute(config)?;
            }
            Self::Labels { subcommand } => {
                subcommand.execute(config)?;
            }
            Self::List {
                ids,
                file,
                title,
                authors,
                tags,
                labels,
                deleted,
                output,
            } => {
                let mut repo = load_repo(config)?;
                let papers = repo.list(
                    ids.unwrap_or_default().0,
                    file,
                    title,
                    authors,
                    tags,
                    labels,
                    deleted,
                )?;

                match output {
                    OutputStyle::Table => {
                        let table = Table::from(papers);
                        println!("{table}");
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
                let mut repo = load_repo(config)?;
                let note = repo.get_note(paper_id)?;

                let content = match &note {
                    Some(note) => note.content.clone(),
                    None => match &config.notes_template {
                        PathOrString::File(template_file) => {
                            let abs_path = if template_file.is_absolute() {
                                template_file.clone()
                            } else {
                                repo.root().join(template_file)
                            };
                            debug!(?abs_path, ?template_file, "New note, loading template");
                            let mut f = File::open(abs_path)?;
                            let mut template = String::new();
                            f.read_to_string(&mut template)?;
                            template
                        }
                        PathOrString::Content(s) => s.clone(),
                    },
                };

                let paper = repo.get_paper(paper_id)?;
                let dont_edit = "# Do not edit this metadata, write notes below.";
                let content = format!(
                    "---\n{}{dont_edit}\n---\n\n{}",
                    serde_yaml::to_string(&paper).unwrap(),
                    content
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

                if let Some(mut note) = note {
                    note.content = result.content;
                    repo.update_note(note)?;
                } else {
                    repo.insert_note(papers_core::db::NewNote {
                        paper_id,
                        content: result.content,
                    })?;
                }
            }
            Self::RenameFiles {
                strategies,
                dry_run,
            } => {
                let mut repo = load_repo(config)?;
                for paper in repo.list(
                    Vec::new(),
                    None,
                    None,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    false,
                )? {
                    if let Some(filename) = &paper.filename {
                        let path = PathBuf::from(filename);
                        if !path.is_file() {
                            error!("{}: Path for paper ({:?}) wasn't a file", paper.id, path);
                            continue;
                        }
                        // the file exists
                        // make the new name and check that file doesn't exist
                        let new_name = strategies.iter().find_map(|s| s.rename(&paper).ok());
                        let new_filename = if let Some(new_name) = new_name {
                            new_name
                        } else {
                            error!("{}: Failed to generate new name for paper", paper.id);
                            continue;
                        };

                        let new_path = if let Some(parent) = path.parent() {
                            parent
                                .join(new_filename)
                                .with_extension(path.extension().unwrap_or_default())
                        } else {
                            PathBuf::from(new_filename)
                                .with_extension(path.extension().unwrap_or_default())
                        };

                        if new_path == path {
                            println!(
                                "{}: Skipping paper as already has the correct name",
                                paper.id
                            );
                            continue;
                        }
                        if new_path.exists() {
                            error!(
                                "{}: New path for paper already exists: {:?}",
                                paper.id, new_path
                            );
                            continue;
                        }

                        // old exists, new doesn't exist, do the rename
                        if !dry_run {
                            rename(&path, &new_path).unwrap();
                            repo.update(paper.id, Some(&new_path), None, None).unwrap();
                        }
                        println!("{}: Renamed {:?} to {:?}", paper.id, path, new_path);
                    } else {
                        debug!(id = paper.id, "Skipping paper");
                        println!("{}: Skipping paper as it has no file associated", paper.id)
                    }
                }
            }
            Self::Open { paper_id } => {
                let mut repo = load_repo(config)?;
                let root = repo.root().to_owned();
                let paper = repo.get_paper(paper_id)?;
                if let Some(filename) = &paper.filename {
                    let path = root.join(filename);
                    info!(?path, "Opening");
                    open::that(path)?;
                } else {
                    info!("No file associated with that paper");
                }
            }
            Self::Completions { shell, dir } => {
                let path = gen_completions(shell, &dir);
                info!(?path, ?shell, "Generated completions");
            }
            Self::Import { file } => {
                let papers = match file {
                    FileOrStdin::File(path) => {
                        let reader = File::open(path)?;
                        let papers: Vec<Paper> = serde_json::from_reader(reader)?;
                        papers
                    }
                    FileOrStdin::Stdin => {
                        let reader = stdin();
                        let papers: Vec<Paper> = serde_json::from_reader(reader)?;
                        papers
                    }
                };
                let mut repo = load_repo(config)?;
                for paper in papers {
                    let id = repo.import(paper)?;
                    info!(id, "Added paper");
                }
            }
        }
        Ok(())
    }
}

fn load_repo(config: &Config) -> anyhow::Result<Repo> {
    let cwd = current_dir()?;
    let repo_dir = if let Ok(repo_dir) = repo::find_root(&cwd, &config.db_filename) {
        debug!(?repo_dir, "Found repo dir from searching.");
        repo_dir
    } else {
        debug!(repo_dir=?config.default_repo, "Did not find repo dir from searching, using default repo.");
        config.default_repo.to_owned()
    };
    let repo = Repo::load(&repo_dir, &config.db_filename)?;
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
    /// Add authors to papers.
    Add {
        /// Ids of papers to add authors to, e.g. 1 1,2 1-3,5.
        #[clap()]
        ids: Ids,

        /// Authors to add.
        #[clap()]
        authors: Vec<Author>,
    },
    /// Remove authors from papers.
    Remove {
        /// Ids of papers to remove authors from, e.g. 1 1,2 1-3,5.
        #[clap()]
        ids: Ids,

        /// Authors to remove.
        #[clap()]
        authors: Vec<Author>,
    },
}

impl AuthorsCommands {
    /// Execute authors commands.
    pub fn execute(self, config: &Config) -> anyhow::Result<()> {
        match self {
            Self::Add { ids, authors } => {
                let mut repo = load_repo(config)?;
                for id in ids.0 {
                    if let Err(err) = repo.add_authors(id, authors.clone()) {
                        warn!(id, %err, "Failed to add authors");
                    }
                }
            }
            Self::Remove { ids, authors } => {
                let mut repo = load_repo(config)?;
                for id in ids.0 {
                    if let Err(err) = repo.remove_authors(id, authors.clone()) {
                        warn!(id, %err, "Failed to remove authors");
                    }
                }
            }
        }
        Ok(())
    }
}

/// Manage tags.
#[derive(Debug, clap::Parser)]
pub enum TagsCommands {
    /// Add tags to papers.
    Add {
        /// Ids of papers to add tags to, e.g. 1 1,2 1-3,5.
        #[clap()]
        ids: Ids,

        /// Tags to add.
        #[clap()]
        tags: Vec<Tag>,
    },
    /// Remove tags from papers.
    Remove {
        /// Ids of papers to remove tags from, e.g. 1 1,2 1-3,5.
        #[clap()]
        ids: Ids,

        /// Tags to remove.
        #[clap()]
        tags: Vec<Tag>,
    },
}

impl TagsCommands {
    /// Execute tags commands.
    pub fn execute(self, config: &Config) -> anyhow::Result<()> {
        match self {
            Self::Add { ids, tags } => {
                let mut repo = load_repo(config)?;
                for id in ids.0 {
                    if let Err(err) = repo.add_tags(id, tags.clone()) {
                        warn!(id, %err, "Failed to add tags");
                    }
                }
            }
            Self::Remove { ids, tags } => {
                let mut repo = load_repo(config)?;
                for id in ids.0 {
                    if let Err(err) = repo.remove_tags(id, tags.clone()) {
                        warn!(id, %err, "Failed to remove tags");
                    }
                }
            }
        }
        Ok(())
    }
}

/// Manage labels.
#[derive(Debug, clap::Parser)]
pub enum LabelsCommands {
    /// Add labels to papers.
    Add {
        /// Ids of papers to add labels to, e.g. 1 1,2 1-3,5.
        #[clap()]
        ids: Ids,

        /// Labels to add.
        #[clap()]
        labels: Vec<Label>,
    },
    /// Remove labels from papers.
    Remove {
        /// Ids of papers to remove labels from, e.g. 1 1,2 1-3,5.
        #[clap()]
        ids: Ids,

        /// Labels to remove.
        #[clap()]
        labels: Vec<Tag>,
    },
}

impl LabelsCommands {
    /// Execute label commands.
    pub fn execute(self, config: &Config) -> anyhow::Result<()> {
        match self {
            Self::Add { ids, labels } => {
                let mut repo = load_repo(config)?;
                for id in ids.0 {
                    if let Err(err) = repo.add_labels(id, labels.clone()) {
                        warn!(id, %err, "Failed to add labels");
                    }
                }
            }
            Self::Remove { ids, labels } => {
                let mut repo = load_repo(config)?;
                for id in ids.0 {
                    if let Err(err) = repo.remove_labels(id, labels.clone()) {
                        warn!(id, %err, "Failed to remove labels");
                    }
                }
            }
        }
        Ok(())
    }
}

/// Fetch a url to a local file, returning the path to the fetch file.
fn fetch_url(url: &Url, path: &Path) -> anyhow::Result<PathBuf> {
    let mut filename = path.to_owned();

    if filename.exists() {
        warn!(?filename, "Path already exists, try moving it");
    }

    debug!(user_agent = APP_USER_AGENT, "Building http client");
    let client = match reqwest::blocking::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            warn!(%err,"Failed to create http client.");
            return Err(err.into());
        }
    };
    info!(%url, "Fetching");
    let mut res = match client
        .get(url.clone())
        .send()
        .expect("Failed to get url")
        .error_for_status()
    {
        Ok(res) => res,
        Err(err) => {
            warn!(%err, %url, "Failed to get resource.");
            return Err(err.into());
        }
    };
    let headers = res.headers();
    if let Some(content_type) = headers.get(http::header::CONTENT_TYPE) {
        if content_type == "application/pdf" {
            // ensure the path ends in pdf
            if let Some("pdf") = filename.extension().and_then(|s| s.to_str()) {
                debug!(?filename, "Filename already has pdf extension");
            } else {
                debug!(?filename, "Setting pdf extension on filename");
                filename.set_extension("pdf");
            }
        } else {
            warn!(
                ?content_type,
                "File fetched was not a pdf, perhaps it needs authorisation?"
            )
        }
    }

    let mut file = match File::create(&filename) {
        Ok(file) => file,
        Err(err) => {
            warn!(%err, ?filename,"Failed to create file");
            return Err(err.into());
        }
    };
    debug!(%url, ?filename, "Saving");
    match std::io::copy(&mut res, &mut file) {
        Ok(_) => {}
        Err(err) => {
            warn!(%err, ?filename, "Failed to copy from http response to file");
            return Err(err.into());
        }
    };
    info!(%url, ?filename, "Fetched");
    Ok(filename)
}

fn add<P: AsRef<Path>>(
    repo: &mut Repo,
    file: Option<P>,
    url: Option<String>,
    title: Option<String>,
    authors: BTreeSet<Author>,
    tags: BTreeSet<Tag>,
    labels: BTreeSet<Label>,
) -> anyhow::Result<Paper> {
    if let Some(file) = file.as_ref() {
        let file = file.as_ref();
        if !file.is_file() {
            anyhow::bail!("Path was not a file: {:?}", file);
        }
    }

    let paper = repo.add(file, url, title, authors, tags, labels)?;
    info!(id = paper.id, filename = paper.filename, "Added paper");

    Ok(paper)
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
                        return Some(found_title.trim().to_owned());
                    }
                }
            }
        }
    }
    warn!("Couldn't find a title in pdf metadata");
    None
}

fn extract_authors(file: &Path) -> BTreeSet<Author> {
    match pdf::file::File::<Vec<u8>>::open(file) {
        Ok(pdf_file) => {
            debug!(?file, "Loaded pdf file");
            if let Some(info) = pdf_file.trailer.info_dict.as_ref() {
                debug!(?file, ?info, "Found the info dict");
                // try and extract the authors
                if let Some(found_authors) = info.get("Author") {
                    debug!(?file, ?found_authors, "Found authors");
                    match found_authors.as_string().and_then(|ft| ft.as_str()) {
                        Ok(found_authors) => {
                            if !found_authors.is_empty() {
                                debug!(?file, ?found_authors, "Setting auto authors");
                                return found_authors
                                    .split(|c: char| {
                                        // names can have alphabet, whitespace or full stops e.g.
                                        // First M. Last
                                        !c.is_alphanumeric() && !c.is_whitespace() && c != '.'
                                    })
                                    .map(|a| a.trim())
                                    .filter(|s| !s.is_empty())
                                    .map(Author::new)
                                    .collect();
                            } else {
                                debug!("Authors was empty");
                            }
                        }
                        Err(err) => {
                            debug!(%err, ?found_authors, "Failed to get authors field as string");
                        }
                    }
                }
            }
        }
        Err(err) => {
            debug!(%err, "Failed to open pdf file");
        }
    }
    warn!("Couldn't find authors in pdf metadata");
    BTreeSet::new()
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
