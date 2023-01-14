mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "add --help",
        expect![[r#"
            Add a paper document from a url or local file and add it to the repo

            Usage: papers add [OPTIONS] <URL_OR_PATH>

            Arguments:
              <URL_OR_PATH>  Url to fetch from or path of a local file in the repo

            Options:
                  --name <NAME>      Name of the file to save it to. Defaults to the basename of the url
                  --title <TITLE>    Title of the file
              -a, --author <author>  Authors to associate with this file
              -t, --tag <tag>        Tags to associate with this file
              -l, --label <label>    Labels to associate with this file. Labels take the form `key=value`
              -h, --help             Print help information"#]],
        expect![""],
    );
}
