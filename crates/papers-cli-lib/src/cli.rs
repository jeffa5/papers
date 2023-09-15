use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{read_dir, rename, File},
    io::{stdin, stdout},
    path::{Path, PathBuf},
};

use clap::{CommandFactory, ValueEnum};
use clap_complete::{generate_to, Generator, Shell};
use papers_core::{author::Author, paper::Paper, repo::Repo, tag::Tag};
use pdf::file::FileOptions;
use reqwest::Url;
use tracing::{debug, info, warn};

use papers_core::label::Label;

use crate::{
    config::Config,
    fuzzy::select_paper,
    interactive::{input, input_bool, input_default, input_opt, input_vec, input_vec_default},
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

    /// Commands.
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

/// Subcommands for the cli.
#[derive(Debug, clap::Subcommand)]
pub enum SubCommand {
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
    /// List the papers stored with this repo.
    List {
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
    /// Automatically rename files to match their entry in the database.
    RenameFiles {
        /// Strategy to use in renaming.
        #[clap(required = true)]
        strategies: Vec<rename_files::Strategy>,

        /// Print information but don't perform renaming.
        #[clap(long)]
        dry_run: bool,
    },
    /// Open the pdf file for the given paper.
    Open {
        /// Id of the paper to open, fuzzy selected if not given.
        #[clap()]
        path: Option<PathBuf>,
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

    /// Check consistency of things in the repo.
    Doctor {
        /// Try and fix the problems
        #[clap(long)]
        fix: bool,
    },
}

impl SubCommand {
    /// Execute a subcommand.
    pub fn execute(self, config: &Config) -> anyhow::Result<()> {
        match self {
            Self::Add {
                mut url,
                mut fetch,
                mut file,
                title,
                mut authors,
                mut tags,
                mut labels,
            } => {
                let mut repo = load_repo(config)?;
                let mut new_title;
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
                            fetch = Some(input_bool(&format!("Fetch {}", url), true));
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
                            } else {
                                anyhow::bail!("No file to downlod to");
                            }
                        }
                    }

                    new_title = if let Some(title) = &title {
                        println!("Using title {}", title);
                        title.clone()
                    } else {
                        let extracted_title = if let Some(file) = &file {
                            extract_title(file)
                        } else {
                            None
                        };
                        if let Some(extracted_title) = extracted_title {
                            input_default("Title", &extracted_title)
                        } else {
                            input("Title")
                        }
                    };

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
                            authors = input_vec_default("Authors", ",", &extracted_authors_str);
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
                    new_title = title.unwrap_or_default();

                    if let Some(file) = &file {
                        if new_title.is_empty() {
                            new_title = extract_title(file).unwrap_or_default();
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
                    new_title,
                    authors.clone(),
                    tags.clone(),
                    labels.clone(),
                ) {
                    Ok(paper) => {
                        println!("Added paper {}", paper.title);
                    }
                    Err(err) => {
                        warn!(%err, "Failed to add paper");
                        error!("Failed to add paper: {}", err);
                    }
                }
            }
            Self::List {
                file,
                title,
                authors,
                tags,
                labels,
                output,
            } => {
                let mut repo = load_repo(config)?;
                let papers = repo
                    .list(file, title, authors, tags, labels)?
                    .into_iter()
                    .map(|p| p.1)
                    .collect::<Vec<_>>();

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
            Self::RenameFiles {
                strategies,
                dry_run,
            } => {
                let mut repo = load_repo(config)?;
                let root = repo.root().to_owned();
                for (paper_path, paper) in repo.all_papers() {
                    let new_name = strategies.iter().find_map(|s| s.rename(&paper).ok());
                    let new_name = if let Some(new_name) = new_name {
                        new_name
                    } else {
                        error!("Failed to generate new name for paper");
                        continue;
                    };

                    if let Some(filename) = &paper.filename {
                        let path = root.join(filename);
                        if path.is_file() {
                            let new_extension = if let Ok(Some(kind)) = infer::get_from_path(&path)
                            {
                                debug!(?path, ?kind, "Detected filetype");
                                kind.extension()
                            } else {
                                debug!(?path, "Failed to detect filetype");
                                path.extension().unwrap_or_default().to_str().unwrap()
                            };

                            // the file exists
                            // make the new name and check that file doesn't exist

                            let new_path = if let Some(parent) = path.parent() {
                                parent.join(&new_name).with_extension(new_extension)
                            } else {
                                PathBuf::from(&new_name).with_extension(new_extension)
                            };

                            if new_path != path {
                                if !new_path.exists() {
                                    // old exists, new doesn't exist, do the rename
                                    println!("Renaming {path:?} to {new_path:?}");
                                    if !dry_run {
                                        rename(&path, &new_path).unwrap();
                                        repo.update(&paper_path, Some(&new_path)).unwrap();
                                    }
                                }
                            }
                        }
                    } else {
                        debug!("Skipping paper");
                    }

                    let new_paper_path = root.join(new_name).with_extension("md");
                    let paper_path = root.join(paper_path);
                    if !new_paper_path.exists() {
                        if paper_path != new_paper_path {
                            println!("Renaming {paper_path:?} to {new_paper_path:?}");
                            if !dry_run {
                                rename(&paper_path, new_paper_path).unwrap();
                            }
                        }
                    }
                }
            }
            Self::Open { path } => {
                let repo = load_repo(config)?;
                let root = repo.root().to_owned();

                let path = match path {
                    Some(path) => path,
                    None => {
                        let all_papers = repo.all_papers().into_iter().map(|n| n.1).collect();

                        match select_paper(all_papers) {
                            Some(p) => repo.get_path(&p),
                            None => {
                                anyhow::bail!("No paper selected");
                            }
                        }
                    }
                };

                let paper = repo.get_paper(&path)?;
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
                    repo.import(paper)?;
                    info!("Added paper");
                }
            }
            Self::Doctor { fix } => {
                let repo = load_repo(config)?;
                let root = repo.root();
                let entries = read_dir(&root)?;
                let mut other_files = BTreeMap::new();
                let mut paths = Vec::new();
                for entry in entries {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() {
                        paths.push(path);
                    }
                }
                paths.sort();

                for path in paths {
                    if path.extension().and_then(|e| e.to_str()) == Some("md") {
                        let paper = repo.get_paper(&path)?;
                        let expected_path = repo.get_path(&paper);
                        let current_path = path.strip_prefix(&root).unwrap();
                        debug!(?expected_path, ?current_path, "Checking paper path");
                        // check that the paper notes are at the right location
                        if expected_path != current_path {
                            warn!(?current_path, ?expected_path, "Paper notes at wrong path");
                            if fix {
                                info!(?current_path, ?expected_path, "Moving paper notes");
                                rename(root.join(current_path), root.join(expected_path))?;
                            }
                        }

                        // check that the paper's file exists
                        if let Some(filename) = paper.filename {
                            let abs_filename = root.join(&filename);
                            if !abs_filename.is_file() {
                                warn!(
                                    ?current_path,
                                    ?filename,
                                    "File is not at the named location"
                                );
                            } else {
                                other_files.insert(filename, true);
                            }
                        }
                    } else {
                        other_files
                            .entry(
                                path.strip_prefix(root)
                                    .unwrap()
                                    .to_string_lossy()
                                    .into_owned(),
                            )
                            .or_default();
                    }
                }

                for (path, matched) in other_files {
                    if !matched {
                        warn!(?path, "Found unmatched file");
                    }
                }
            }
        }
        Ok(())
    }
}

fn load_repo(config: &Config) -> anyhow::Result<Repo> {
    debug!(repo_dir=?config.default_repo, "Using default repo.");
    let repo_dir = config.default_repo.to_owned();
    let repo = Repo::load(&repo_dir)?;
    Ok(repo)
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
    title: String,
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
    info!(filename = paper.filename, "Added paper");

    Ok(paper)
}

fn extract_title(file: &Path) -> Option<String> {
    if let Ok(pdf_file) = FileOptions::cached().open(file) {
        debug!(?file, "Loaded pdf file");
        if let Some(info) = pdf_file.trailer.info_dict.as_ref() {
            debug!(?file, ?info, "Found the info dict");
            // try and extract the title
            if let Some(found_title) = info.get("Title") {
                debug!(?file, "Found title");
                if let Ok(found_title) = found_title
                    .as_string()
                    .map(|ft| ft.to_string().unwrap_or_default())
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
    match FileOptions::cached().open(file) {
        Ok(pdf_file) => {
            debug!(?file, "Loaded pdf file");
            if let Some(info) = pdf_file.trailer.info_dict.as_ref() {
                debug!(?file, ?info, "Found the info dict");
                // try and extract the authors
                if let Some(found_authors) = info.get("Author") {
                    debug!(?file, ?found_authors, "Found authors");
                    match found_authors.as_string().and_then(|ft| ft.to_string()) {
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
