mod common;
use common::check_ok;

use expect_test::expect;

#[test]
fn test_help() {
    check_ok(
        "show --help",
        expect![[r#"
        Show all information about a paper

        Usage: papers show [OPTIONS] <IDS>

        Arguments:
          <IDS>
                  Ids of papers to show information for, e.g. 1 1,2 1-3,5

        Options:
          -o, --output <OUTPUT>
                  Output the paper in different formats

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
