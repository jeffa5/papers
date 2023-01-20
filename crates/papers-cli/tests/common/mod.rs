use papers_cli_lib::config::{Config, PaperDefaults};
use std::fs::create_dir_all;
use std::io::Write;
use std::process::{Output, Stdio};
use std::{fs::File, path::PathBuf, process::Command, str::from_utf8};
use tempfile::{tempdir, TempDir};

use expect_test::Expect;

#[derive(Debug)]
pub struct Fixture {
    root: TempDir,
    do_init: bool,
    initialised: bool,
    debug: bool,
}

impl Fixture {
    pub fn new() -> Self {
        let s = Self {
            root: tempdir().unwrap(),
            do_init: true,
            initialised: false,
            debug: false,
        };

        create_dir_all(s.root_dir()).unwrap();

        let config_path = s.config_path();
        let config_file = File::create(&config_path).unwrap();
        serde_yaml::to_writer(config_file, &s.config()).unwrap();

        let file1_path = s.root_dir().join("file1.pdf");
        let mut file1 = File::create(&file1_path).unwrap();
        writeln!(file1, "test pdf").unwrap();

        let nested_path = s.root_dir().join("nested");
        create_dir_all(&nested_path).unwrap();
        let nested_file1_path = nested_path.join("file1.pdf");
        let mut nested_file1 = File::create(&nested_file1_path).unwrap();
        writeln!(nested_file1, "test pdf").unwrap();

        let neighbour_path = s.root.path().join("neighbour");
        create_dir_all(&neighbour_path).unwrap();

        let neighbour_file1_path = neighbour_path.join("file1.pdf");
        let mut neighbour_file1 = File::create(&neighbour_file1_path).unwrap();
        writeln!(neighbour_file1, "test pdf").unwrap();

        s
    }

    #[allow(dead_code)]
    pub fn no_init(&mut self) {
        self.do_init = false;
    }

    #[allow(dead_code)]
    pub fn debug(&mut self) {
        self.debug = true;
    }

    pub fn root_dir(&self) -> PathBuf {
        self.root.path().join("root")
    }

    pub fn config(&self) -> Config {
        Config {
            db_filename: "test.db".into(),
            default_repo: self.root.path().to_owned(),
            notes_template: None,
            paper_defaults: PaperDefaults::default(),
        }
    }

    pub fn config_path(&self) -> PathBuf {
        self.root_dir().join("config.yaml")
    }

    pub fn run_with_stdin(&self, args: &str, stdin: &str) -> Output {
        let args = format!(
            "{} --config-file {}",
            args,
            self.config_path().to_string_lossy()
        );

        let exe = PathBuf::from(env!("CARGO_BIN_EXE_papers"));
        println!("Found exe: {:?}", exe);
        println!("Using dir: {:?}", self.root_dir());
        let mut cmd = Command::new(exe);
        cmd.args(args.split_whitespace())
            .current_dir(self.root_dir());
        if self.debug {
            cmd.env("RUST_LOG", "debug");
        }
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        println!("Running command: {:?}", cmd);
        let mut child = cmd.spawn().unwrap();
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(stdin.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }

    pub fn run(&self, args: &str) -> Output {
        self.run_with_stdin(args, "")
    }

    pub fn check_ok_with_stdin(&mut self, args: &str, stdin: &str, out: Expect, err: Expect) {
        if self.do_init && !self.initialised {
            self.run("init");
            self.initialised = true;
        }

        let output = self.run_with_stdin(args, stdin);
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
        out.assert_eq(&stdout);
        err.assert_eq(&stderr);
    }

    pub fn check_ok(&mut self, args: &str, out: Expect, err: Expect) {
        self.check_ok_with_stdin(args, "", out, err)
    }
}
