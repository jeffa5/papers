mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "open --help",
        expect![[r#"
            Open the file for the given paper

            Usage: papers open <PAPER_ID>

            Arguments:
              <PAPER_ID>  Id of the paper to open

            Options:
              -h, --help  Print help information"#]],
        expect![""],
    );
}
