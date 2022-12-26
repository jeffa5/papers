use clap_complete::{shells, Generator};
use std::env;
use std::fs::create_dir_all;
use std::io::Error;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };
    let share_dir = PathBuf::from(outdir).join("share");

    create_dir_all(&share_dir).unwrap();

    gen_completions(shells::Bash, &share_dir);
    gen_completions(shells::Zsh, &share_dir);
    gen_completions(shells::Fish, &share_dir);

    Ok(())
}

fn gen_completions<S>(shell: S, share_dir: &Path)
where
    S: Generator,
{
    let path = papers_cli_lib::cli::gen_completions(shell, share_dir).unwrap();
    println!("cargo:warning=completion file is generated: {:?}", path);
}
