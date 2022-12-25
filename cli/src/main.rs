use clap::Parser;
use directories::ProjectDirs;
use tracing::debug;

use papers_cli_lib::cli::Cli;
use papers_cli_lib::config::Config;

fn main() -> anyhow::Result<()> {
    let options = Cli::parse();
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
