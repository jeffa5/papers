use clap::Parser;
use tracing::debug;

mod cli;
mod config;

fn main() {
    let options = cli::Cli::parse();
    tracing_subscriber::fmt::init();
    debug!(?options, "Parsed options");
    options.cmd.execute();
}
