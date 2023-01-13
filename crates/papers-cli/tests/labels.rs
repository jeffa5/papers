mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "labels --help",
        expect![[r#"
            Manage labels associated with a paper

            Usage: papers labels <COMMAND>

            Commands:
              add     Add labels to a paper
              remove  Remove labels from a paper
              help    Print this message or the help of the given subcommand(s)

            Options:
              -h, --help  Print help information"#]],
        expect![""],
    );
}
