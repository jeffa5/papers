mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "tags --help",
        expect![[r#"
            Manage tags associated with a paper

            Usage: papers tags [OPTIONS] <COMMAND>

            Commands:
              add     Add tags to papers
              remove  Remove tags from papers
              help    Print this message or the help of the given subcommand(s)

            Options:
                  --db-filename <DB_FILENAME>  Filename for the database
              -h, --help                       Print help information"#]],
        expect![""],
    );
}
