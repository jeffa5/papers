mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "authors --help",
        expect![[r#"
            Manage authors associated with a paper

            Usage: papers authors <COMMAND>

            Commands:
              add     Add authors to papers
              remove  Remove authors from papers
              help    Print this message or the help of the given subcommand(s)

            Options:
              -h, --help  Print help information"#]],
        expect![""],
    );
}
