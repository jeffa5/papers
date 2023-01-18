mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "open --help",
        expect![[r#"
            Open the file for the given paper

            Usage: papers open [OPTIONS] <PAPER_ID>

            Arguments:
              <PAPER_ID>  Id of the paper to open

            Options:
                  --default-repo <DEFAULT_REPO>  Default repo to use if not found in parents of current directory
                  --db-filename <DB_FILENAME>    Filename for the database
              -h, --help                         Print help information"#]],
        expect![""],
    );
}
