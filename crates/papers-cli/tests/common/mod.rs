use std::{path::PathBuf, process::Command, str::from_utf8};

use expect_test::Expect;

pub fn check_ok(args: &str, expect_out: Expect, expect_err: Expect) {
    let exe = PathBuf::from(env!("CARGO_BIN_EXE_papers"));
    let output = Command::new(exe)
        .args(args.split_whitespace())
        .output()
        .unwrap();
    let stdout = from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .map(|s| s.trim_end().to_owned())
        .collect::<Vec<String>>()
        .join("\n");
    let stderr = from_utf8(&output.stderr)
        .unwrap()
        .lines()
        .map(|s| s.trim_end().to_owned())
        .collect::<Vec<String>>()
        .join("\n");
    expect_out.assert_eq(&stdout);
    expect_err.assert_eq(&stderr);
}
