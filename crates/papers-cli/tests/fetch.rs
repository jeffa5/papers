mod common;
use common::check_ok;
use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "fetch --help",
        expect![[r#"
            Fetch a paper pdf from a url and add it to the repo

            Usage: papers fetch [OPTIONS] <URL> [NAME]

            Arguments:
              <URL>   Url to fetch the pdf from
              [NAME]  Name of the file to save it to. Defaults to the basename of the url

            Options:
                  --title <TITLE>    Title of the file
              -a, --author <author>  Authors to associate with this file
              -t, --tag <tag>        Tags to associate with this file
              -l, --label <label>    Labels to associate with this file. Labels take the form `key=value`
              -h, --help             Print help information"#]],
        expect![""],
    );
}
