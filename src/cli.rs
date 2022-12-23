use std::{env::current_dir, path::PathBuf};

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
    Fetch {},
    Add {
        #[clap()]
        file: PathBuf,
    },
    List {},
    Search {},
}

impl SubCommand {
    pub fn execute(&self) {
        match self {
            SubCommand::Init {} => {
                let cwd = current_dir().unwrap();
                Repo::init(&cwd);
                info!("Initialised the current directory")
            }
            SubCommand::Fetch {} => {
                todo!()
            }
            SubCommand::Add { file } => {
                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                repo.add(file);
                info!("Added {:?}", file);
            }
            SubCommand::List {} => {
                let cwd = current_dir().unwrap();
                let mut repo = Repo::load(&cwd);
                let papers = repo.list();
                for paper in papers {
                    println!("{:?}", paper);
                }
            }
            SubCommand::Search {} => todo!(),
        }
    }
}
