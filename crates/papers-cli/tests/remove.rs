mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "remove --help",
        expect![[r#"
            Remove papers from being tracked

            Usage: papers remove [OPTIONS] <IDS>

            Arguments:
              <IDS>  Ids of papers to remove, e.g. 1 1,2 1-3,5

            Options:
                  --with-file                  Also remove the paper file
                  --db-filename <DB_FILENAME>  Filename for the database
              -h, --help                       Print help information"#]],
        expect![[""]],
    );
}
