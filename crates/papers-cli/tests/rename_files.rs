mod common;
use common::Fixture;
use expect_test::expect;

#[test]
fn test_help() {
    let mut f = Fixture::new();
    f.check_ok(
        "rename-files --help",
        expect![[r#"
            Automatically rename files to match their entry in the database

            Usage: papers rename-files [OPTIONS] <STRATEGIES>...

            Arguments:
              <STRATEGIES>...
                      Strategy to use in renaming

                      Possible values:
                      - title: Rename to match the title of the paper
                      - id:    Rename to match the id of the paper

            Options:
              -c, --config-file <CONFIG_FILE>
                      Config file path to load

                  --dry-run
                      Print information but don't perform renaming

                  --default-repo <DEFAULT_REPO>
                      Default repo to use if not found in parents of current directory

                  --db-filename <DB_FILENAME>
                      Filename for the database

              -h, --help
                      Print help (see a summary with '-h')"#]],
        expect![""],
    );
}
