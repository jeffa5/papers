mod common;
use common::Fixture;
use expect_test::expect;

#[test]
fn test_help() {
    let mut f = Fixture::new();
    f.check_ok(
        "review --help",
        expect![[r#"
            Review papers that have been unseen too long

            Usage: papers review [OPTIONS]

            Options:
              -c, --config-file <CONFIG_FILE>    Config file path to load
                  --open                         Open the pdf file too
                  --default-repo <DEFAULT_REPO>  Default repo to use if not found in parents of current directory
              -h, --help                         Print help"#]],
        expect![""],
    );
}
