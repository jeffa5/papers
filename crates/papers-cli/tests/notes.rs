mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "notes --help",
        expect![[r#"
            Manage notes associated with a paper

            Usage: papers notes [OPTIONS] <PAPER_ID>

            Arguments:
              <PAPER_ID>  Id of the paper to update notes for

            Options:
                  --default-repo <DEFAULT_REPO>  Default repo to use if not found in parents of current directory
                  --db-filename <DB_FILENAME>    Filename for the database
              -h, --help                         Print help information"#]],
        expect![""],
    );
}
