mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "labels --help",
        expect![[r#"
            Manage labels associated with a paper

            Usage: papers labels [OPTIONS] <COMMAND>

            Commands:
              add     Add labels to papers
              remove  Remove labels from papers
              help    Print this message or the help of the given subcommand(s)

            Options:
                  --db-filename <DB_FILENAME>  Filename for the database
              -h, --help                       Print help information"#]],
        expect![""],
    );
}
