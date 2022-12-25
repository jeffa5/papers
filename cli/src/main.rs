use clap::Parser;
use directories::ProjectDirs;
use tracing::debug;

use crate::config::Config;

mod cli;
mod config;

fn main() -> anyhow::Result<()> {
    let options = cli::Cli::parse();
    tracing_subscriber::fmt::init();
    debug!(?options, "Parsed options");

    let config_file = if let Some(config_file) = options.config_file.as_ref() {
        config_file.clone()
    } else if let Some(dirs) = ProjectDirs::from("io", "jeffas", "papers") {
        dirs.config_dir().to_owned()
    } else {
        anyhow::bail!("Failed to make project dirs")
    };
    let config = Config::load(&config_file)?;

    debug!(?config, ?config_file, "Loaded config file");

    options.cmd.execute(&config)?;

    Ok(())
}
