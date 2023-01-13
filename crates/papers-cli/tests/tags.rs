mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "tags --help",
        expect![[r#"
            Manage tags associated with a paper

            Usage: papers tags <COMMAND>

            Commands:
              add     Add tags to a paper
              remove  Remove tags from a paper
              help    Print this message or the help of the given subcommand(s)

            Options:
              -h, --help  Print help information"#]],
        expect![""],
    );
}
