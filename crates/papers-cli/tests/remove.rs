mod common;
use common::Fixture;
use expect_test::expect;

#[test]
fn test_help() {
    let mut f = Fixture::new();
    f. check_ok(
        "remove --help",
        expect![[r#"
            Remove papers from being tracked

            Usage: papers remove [OPTIONS] <IDS>

            Arguments:
              <IDS>  Ids of papers to remove, e.g. 1 1,2 1-3,5

            Options:
              -c, --config-file <CONFIG_FILE>    Config file path to load
                  --with-file                    Also remove the paper file
                  --default-repo <DEFAULT_REPO>  Default repo to use if not found in parents of current directory
                  --db-filename <DB_FILENAME>    Filename for the database
              -h, --help                         Print help"#]],
        expect![[""]],
    );
}
