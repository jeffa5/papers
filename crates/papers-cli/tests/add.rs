mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "add --help",
        expect![[r#"
            Add a pdf from a local file to the repo

            Usage: papers add [OPTIONS] <FILE>

            Arguments:
              <FILE>  File to add

            Options:
                  --title <TITLE>    Title of the file
              -a, --author <author>  Authors to associate with this file
              -t, --tag <tag>        Tags to associate with this file
              -l, --label <label>    Labels to associate with this file. Labels take the form `key=value`
              -h, --help             Print help information"#]],
        expect![""],
    );
}
