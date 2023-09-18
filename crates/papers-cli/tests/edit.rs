mod common;
use common::Fixture;
use expect_test::expect;

#[test]
fn test_help() {
    let mut f = Fixture::new();
    f.check_ok(
        "edit --help",
        expect![[r#"
            Edit the notes file for a paper

            Usage: papers edit [OPTIONS] [PATH]

            Arguments:
              [PATH]  Path of the paper to edit, fuzzy selected if not given

            Options:
              -c, --config-file <CONFIG_FILE>    Config file path to load
                  --open                         Open the pdf file too
                  --default-repo <DEFAULT_REPO>  Default repo to use if not found in parents of current directory
              -h, --help                         Print help"#]],
        expect![""],
    );
}
