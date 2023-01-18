mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "--help",
        expect![[r#"
            A paper management program

            Usage: papers [OPTIONS] <COMMAND>

            Commands:
              init         Initialise a new paper repository
              add          Add paper documents from a url or local file
              update       Update metadata about an existing paper
              remove       Remove papers from being tracked
              authors      Manage authors associated with a paper
              tags         Manage tags associated with a paper
              labels       Manage labels associated with a paper
              list         List the papers stored with this repo
              notes        Manage notes associated with a paper
              open         Open the file for the given paper
              completions  Generate cli completion files
              import       Import a list of tasks in json format
              help         Print this message or the help of the given subcommand(s)

            Options:
              -c, --config-file <CONFIG_FILE>  Config file path to load
                  --db-filename <DB_FILENAME>  Filename for the database
              -h, --help                       Print help information"#]],
        expect![""],
    );
}
