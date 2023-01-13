mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "init --help",
        expect![[r#"
            Initialise a new paper repository

            Usage: papers init

            Options:
              -h, --help  Print help information"#]],
        expect![""],
    );
}
