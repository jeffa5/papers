use clap::Parser;
use directories::ProjectDirs;
use std::io;
use tracing::debug;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use papers_cli_lib::cli::Cli;
use papers_cli_lib::config::Config;

fn main() -> anyhow::Result<()> {
    let options = Cli::parse();
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(io::stderr)
        .finish();
    subscriber.init();

    debug!(?options, "Parsed options");

    let config_file = if let Some(config_file) = options.config_file.as_ref() {
        config_file.clone()
    } else if let Some(dirs) = ProjectDirs::from("io", "jeffas", "papers") {
        dirs.config_dir().to_owned()
    } else {
        anyhow::bail!("Failed to make project dirs")
    };
    let mut config = Config::load(&config_file)?;

    // override config values from cli
    if let Some(db_filename) = options.db_filename {
        config.db_filename = db_filename;
    }
    if let Some(default_repo) = options.default_repo {
        config.default_repo = default_repo;
    }

    debug!(?config, ?config_file, "Loaded config file");

    options.cmd.execute(&config)?;

    Ok(())
}
