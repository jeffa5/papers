mod common;
use common::check_ok;

use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "update --help",
        expect![[r#"
            Update metadata about an existing paper

            Usage: papers update [OPTIONS] <IDS>

            Arguments:
              <IDS>  Ids of papers to update, e.g. 1 1,2 1-3,5

            Options:
              -u, --url <URL>                    Url the paper was fetched from
                  --default-repo <DEFAULT_REPO>  Default repo to use if not found in parents of current directory
              -f, --file <FILE>                  File to add
                  --db-filename <DB_FILENAME>    Filename for the database
                  --title <TITLE>                Title of the file
              -h, --help                         Print help information"#]],
        expect![""],
    );
}
