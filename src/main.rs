use clap::Parser;
use directories::ProjectDirs;
use tracing::debug;

use crate::config::Config;

mod cli;
mod config;

fn main() {
    let options = cli::Cli::parse();
    tracing_subscriber::fmt::init();
    debug!(?options, "Parsed options");

    let config_file = if let Some(config_file) = options.config_file.as_ref() {
        config_file.clone()
    } else {
        ProjectDirs::from("io", "jeffas", "papers")
            .unwrap()
            .config_dir()
            .to_owned()
    };
    let config = Config::load(&config_file);

    debug!(?config, ?config_file, "Loaded config file");

    options.cmd.execute(&config);
}
