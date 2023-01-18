mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "init --help",
        expect![[r#"
            Initialise a new paper repository

            Usage: papers init [OPTIONS] [DIR]

            Arguments:
              [DIR]  Directory to initialise [default: .]

            Options:
                  --default-repo <DEFAULT_REPO>  Default repo to use if not found in parents of current directory
                  --db-filename <DB_FILENAME>    Filename for the database
              -h, --help                         Print help information"#]],
        expect![""],
    );
}
