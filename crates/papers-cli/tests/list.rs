mod common;
use common::Fixture;
use expect_test::expect;

#[test]
fn test_help() {
    let mut f = Fixture::new();
    f.check_ok(
        "list --help",
        expect![[r#"
            List the papers stored with this repo

            Usage: papers list [OPTIONS] [IDS]

            Arguments:
              [IDS]
                      Paper ids to filter to, e.g. 1 1,2 1-3,5

            Options:
              -c, --config-file <CONFIG_FILE>
                      Config file path to load

              -f, --file <FILE>
                      Filter down to papers that have filenames which match this (case-insensitive)

                  --default-repo <DEFAULT_REPO>
                      Default repo to use if not found in parents of current directory

                  --title <TITLE>
                      Filter down to papers whose titles match this (case-insensitive)

              -a, --author <author>
                      Filter down to papers that have all of the given authors

                  --db-filename <DB_FILENAME>
                      Filename for the database

              -t, --tag <tag>
                      Filter down to papers that have all of the given tags

              -l, --label <label>
                      Filter down to papers that have all of the given labels. Labels take the form `key=value`

                  --deleted
                      Show papers that have been deleted too

              -o, --output <OUTPUT>
                      Output the filtered selection of papers in different formats

                      [default: table]

                      Possible values:
                      - table: Pretty table format
                      - json:  Json format
                      - yaml:  Yaml format

              -h, --help
                      Print help information (use `-h` for a summary)"#]],
        expect![""],
    );
}
