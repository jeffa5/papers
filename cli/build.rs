use clap_complete::{shells, Generator};
use std::env;
use std::ffi::OsString;
use std::io::Error;

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    gen_completions(shells::Bash, &outdir);
    gen_completions(shells::Zsh, &outdir);
    gen_completions(shells::Fish, &outdir);

    Ok(())
}

fn gen_completions<S>(shell: S, outdir: &OsString)
where
    S: Generator,
{
    let path = papers_cli_lib::cli::gen_completions(shell, outdir);
    println!("cargo:warning=completion file is generated: {:?}", path);
}
