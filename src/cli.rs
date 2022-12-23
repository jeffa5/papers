use std::{env::current_dir, fs::File, path::PathBuf};

use papers::repo::Repo;
use tracing::info;

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[clap(long, short)]
    pub config_file: Option<PathBuf>,

    #[clap(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum SubCommand {
    Init {},
    Fetch {
        #[clap()]
        url: String,

        #[clap()]
        name: Option<String>,

        #[clap(long)]
        title: Option<String>,

        #[clap(name = "tag", long, short)]
        tags: Vec<String>,
    },
    Add {
        #[clap()]
        file: PathBuf,

        #[clap(long)]
        title: Option<String>,

        #[clap(name = "tag", long, short)]
        tags: Vec<String>,
    },
    List {
        #[clap(name = "tag", long, short)]
        tags: Vec<String>,
    },
    Search {},
}

impl SubCommand {
    pub fn execute(self) {
        match self {
            SubCommand::Init {} => {
                let cwd = current_dir().unwrap();
                Repo::init(&cwd);
                info!("Initialised the current directory")
            }
            SubCommand::Fetch { url, name, title,tags } => {
                let mut res = reqwest::blocking::get(&url).expect("Failed to get url");
                let filename = if let Some(name) = name {
                    name
                } else {
                    let last = url.split('/').last().as_ref().unwrap().to_string();
                    last.trim_end_matches(".pdf").to_owned()
                };
                let mut file = File::create(&filename).unwrap();
                std::io::copy(&mut res, &mut file).unwrap();

                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                repo.add(&filename, Some(url), title, tags);
                info!("Added {:?}", filename);
            }
            SubCommand::Add { file,title, tags } => {
                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                repo.add(&file, None, title, tags);
                info!("Added {:?}", file);
            }
            SubCommand::List { tags } => {
                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                let papers = repo.list(tags);
                for paper in papers {
                    println!("{:?}", paper);
                }
            }
            SubCommand::Search {} => todo!(),
        }
    }
}
