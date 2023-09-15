mod common;
use common::Fixture;
use expect_test::expect;

#[test]
fn test_help() {
    let mut f = Fixture::new();
    f.check_ok(
        "--help",
        expect![[r#"
            A paper management program

            Usage: papers [OPTIONS] <COMMAND>

            Commands:
              add           Add a paper to the repo
              list          List the papers stored with this repo
              rename-files  Automatically rename files to match their entry in the database
              open          Open the pdf file for the given paper
              completions   Generate cli completion files
              import        Import a list of tasks in json format
              doctor        Check consistency of things in the repo
              help          Print this message or the help of the given subcommand(s)

            Options:
              -c, --config-file <CONFIG_FILE>    Config file path to load
                  --default-repo <DEFAULT_REPO>  Default repo to use if not found in parents of current directory
              -h, --help                         Print help"#]],
        expect![""],
    );
}
