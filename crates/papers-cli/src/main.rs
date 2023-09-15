use clap::Parser;
use directories::ProjectDirs;
use std::io;
use tracing::debug;
use tracing_subscriber::EnvFilter;

use papers_cli_lib::cli::Cli;
use papers_cli_lib::config::Config;

fn main() -> anyhow::Result<()> {
    let options = Cli::parse();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::builder().from_env().unwrap())
        .with_writer(io::stderr)
        .init();

    debug!(?options, "Parsed options");

    let config_file = if let Some(config_file) = options.config_file.as_ref() {
        config_file.clone()
    } else if let Some(dirs) = ProjectDirs::from("io", "jeffas", "papers") {
        dirs.config_dir().to_owned().join("config.yaml")
    } else {
        anyhow::bail!("Failed to make project dirs")
    };
    let mut config = Config::load(&config_file)?;
    debug!(?config, ?config_file, "Loaded config file");

    if let Some(default_repo) = options.default_repo {
        config.default_repo = default_repo;
    }

    debug!(?config, "Merged config and options");

    options.cmd.execute(&config)?;

    Ok(())
}
