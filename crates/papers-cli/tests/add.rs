mod common;
use common::Fixture;
use expect_test::expect;

#[test]
fn test_help() {
    let mut f = Fixture::new();
    f.check_ok(
        "add --help",
        expect![[r#"
            Add a paper to the repo

            Usage: papers add [OPTIONS]

            Options:
              -c, --config-file <CONFIG_FILE>    Config file path to load
              -u, --url <URL>                    Url to fetch from
                  --default-repo <DEFAULT_REPO>  Default repo to use if not found in parents of current directory
                  --fetch <FETCH>                Whether to fetch the document from URL or not [possible values: true, false]
                  --db-filename <DB_FILENAME>    Filename for the database
              -f, --file <FILE>                  File to add
                  --title <TITLE>                Title of the file
              -a, --author <author>              Authors to associate with these files
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
        "add --file missing.pdf",
        expect![[""]],
        expect!["Error: Not a repo, run `init` first"],
    );
}

#[test]
fn test_add_missing_file() {
    let mut f = Fixture::new();
    f.check_ok("add --file missing.pdf",  expect![[""]], expect![[r#"error: Failed to add paper: Path was not a file: "missing.pdf""#]]);
}

#[test]
fn test_add_present_file() {
    let mut f = Fixture::new();
    f.check_ok(
        "add --file file1.pdf",
        expect!["Added paper 1"],
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
        "add --file nested/file1.pdf",
        expect!["Added paper 1"],
        expect![""],
    );
}

#[test]
fn test_add_file_from_neighbour() {
    let mut f = Fixture::new();
    f.check_ok(
        "add --file ../neighbour/file1.pdf",
        expect![""],
        expect!["error: Failed to add paper: File does not live in the root"],
    );
}

#[test]
fn test_add_interactive() {
    let mut f = Fixture::new();
    f.check_ok_with_stdin(
        "add",
        "",
        expect!["Added paper 1"],
        expect![""],
    );
}
