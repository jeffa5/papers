mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "notes --help",
        expect![[r#"
            Manage notes associated with a paper

            Usage: papers notes <PAPER_ID>

            Arguments:
              <PAPER_ID>  Id of the paper to update notes for

            Options:
              -h, --help  Print help information"#]],
        expect![""],
    );
}
