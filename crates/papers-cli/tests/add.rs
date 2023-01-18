mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "add --help",
        expect![[r#"
            Add paper documents from a url or local file

            Usage: papers add [OPTIONS] [URL_OR_PATH]...

            Arguments:
              [URL_OR_PATH]...  List of Urls to fetch from or paths of local files in the repo

            Options:
                  --title <TITLE>              Title of the file
              -a, --author <author>            Authors to associate with these files
                  --db-filename <DB_FILENAME>  Filename for the database
              -t, --tag <tag>                  Tags to associate with these files
              -l, --label <label>              Labels to associate with these files. Labels take the form `key=value`
              -h, --help                       Print help information"#]],
        expect![""],
    );
}
