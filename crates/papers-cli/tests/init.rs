mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "init --help",
        expect![[r#"
            Initialise a new paper repository

            Usage: papers init [OPTIONS]

            Options:
                  --db-filename <DB_FILENAME>  Filename for the database
              -h, --help                       Print help information"#]],
        expect![""],
    );
}
