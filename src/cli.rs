use std::{env::current_dir, fs::File, path::PathBuf};

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
                repo.add(&filename, Some(url), title, tags, labels);
                info!("Added {:?}", filename);
            }
            SubCommand::Add {
                file,
                title,
                tags,
                labels,
            } => {
                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                repo.add(&file, None, title, tags, labels);
                info!("Added {:?}", file);
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
        }
    }
}
