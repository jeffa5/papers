mod common;
use common::Fixture;
use expect_test::expect;

#[test]
fn test_help() {
    let mut f = Fixture::new();
    f.check_ok(
        "add --help",
        expect![[r#"
            Add paper documents from a url or local file

            Usage: papers add [OPTIONS] [URL_OR_PATH]...

            Arguments:
              [URL_OR_PATH]...  List of Urls to fetch from or paths of local files in the repo

            Options:
              -c, --config-file <CONFIG_FILE>    Config file path to load
                  --title <TITLE>                Title of the file
              -a, --author <author>              Authors to associate with these files
                  --default-repo <DEFAULT_REPO>  Default repo to use if not found in parents of current directory
                  --db-filename <DB_FILENAME>    Filename for the database
              -t, --tag <tag>                    Tags to associate with these files
              -l, --label <label>                Labels to associate with these files. Labels take the form `key=value`
              -h, --help                         Print help information"#]],
        expect![""],
    );
}

#[test]
fn test_add_without_init() {
    let mut f = Fixture::new();
    f.no_init();
    f.check_ok(
        "add missing.pdf",
        expect![[""]],
        expect!["Error: Not a repo, run `init` first"],
    );
}

#[test]
fn test_add_missing_file() {
    let mut f = Fixture::new();
    f.check_ok("add missing.pdf",  expect![[""]], expect![[r#"error: Failed to add paper from file "missing.pdf": Path was not a file: "missing.pdf""#]]);
}

#[test]
fn test_add_present_file() {
    let mut f = Fixture::new();
    f.check_ok(
        "add file1.pdf",
        expect!["Added paper 1 from file1.pdf"],
        expect![""],
    );
}

#[test]
fn test_add_just_title() {
    let mut f = Fixture::new();
    f.check_ok(
        "add --title test-title",
        expect!["Added paper 1"],
        expect![""],
    );
}

#[test]
fn test_add_file_from_nested_dir() {
    let mut f = Fixture::new();
    f.check_ok(
        "add nested/file1.pdf",
        expect!["Added paper 1 from nested/file1.pdf"],
        expect![""],
    );
}

#[test]
fn test_add_file_from_neighbour() {
    let mut f = Fixture::new();
    f.check_ok(
        "add ../neighbour/file1.pdf",
        expect![""],
        expect![[r#"error: Failed to add paper from file "../neighbour/file1.pdf": File does not live in the root"#]],
    );
}
