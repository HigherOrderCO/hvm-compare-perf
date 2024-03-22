#![feature(exit_status_error)]

use std::{
    error::Error,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use regex::Regex;

pub type Result<T> = core::result::Result<T, Box<dyn Error>>;

const MAX_TIME: Duration = Duration::from_secs(120);

// const COMMIT_MODERN: &str = "09a3791cd8194fef28be95305835d4851eb0a854";
// const COMMIT_POST_PTR: &str = "9bdbdcbe0816345545a3adf00704f9f4f01dcfe7";
// const COMMIT_PRE_PTR: &str = "c610b490fb071b7c9891b674bf399addaff3a580";

pub fn main() -> Result<()> {
    let file = fs::read(env!("CARGO_MANIFEST_DIR").to_string() + "/commits.cfg")?;
    let file = std::str::from_utf8(&file)?;
    let commits: Vec<&str> = file.split("\n").collect();
    let mut state = State {
        crate_dir: "./hvm-core".into(),
        git_dir: "./hvm-core".into(),
        re_time: Regex::new("TIME *: *(.+)")?,
        re_rwts: Regex::new("RWTS *: *(.+)")?,
        re_rwps: Regex::new("RPS *: *(.+)")?,
        commits: &commits,
    };
    state.init()?;
    let stats = state.perf_all()?;

    let mut file = File::create("perf.csv")?;
    use std::io::Write;
    writeln!(&mut file, "hash,file,mode,time,rwts,rwps")?;
    for stat in stats {
        writeln!(&mut file, "{}", stat.to_csv())?;
    }

    Ok(())
}

pub struct State<'a> {
    git_dir: PathBuf,
    crate_dir: PathBuf,
    re_time: Regex,
    re_rwts: Regex,
    re_rwps: Regex,
    commits: &'a [&'a str],
}

#[derive(Default, Debug)]
pub struct Stats {
    hash: Option<String>,
    file: Option<String>,
    mode: Option<String>,
    time: Option<String>,
    rwts: Option<String>,
    rwps: Option<String>,
}

impl Stats {
    fn show_short(&self) -> String {
        format!(
            "{time} @ {rwps} rps",
            time = self.time.as_ref().map(|x| x.as_ref()).unwrap_or("???"),
            rwps = &self.rwps.as_ref().map(|x| x.as_ref()).unwrap_or("???")
        )
    }
}
impl Stats {
    fn to_csv(&self) -> String {
        format!(
            "{hash},{file},{mode},{time},{rwts},{rwps}",
            hash = self.hash.as_ref().map(|x| x.as_ref()).unwrap_or(""),
            file = self.file.as_ref().map(|x| x.as_ref()).unwrap_or(""),
            mode = self.mode.as_ref().map(|x| x.as_ref()).unwrap_or(""),
            time = self.time.as_ref().map(|x| x.as_ref()).unwrap_or(""),
            rwts = self.rwts.as_ref().map(|x| x.as_ref()).unwrap_or(""),
            rwps = self.rwps.as_ref().map(|x| x.as_ref()).unwrap_or(""),
        )
    }
}

impl<'a> State<'a> {
    fn init(&mut self) -> Result<()> {
        const GIT_URL: &str = "https://github.com/HigherOrderCO/hvm-core.git";
        /*
        if self.git_dir.exists() && self.git_dir.read_dir()?.count() != 0 {
            fs::remove_dir_all(&self.git_dir)?;
            fs::create_dir_all(&self.git_dir)?;
        }
        */
        if !self.git_dir.exists() {
            fs::create_dir_all(&self.git_dir)?;
        }
        if self.git_dir.read_dir()?.count() == 0 {
            self.create_command_git_clone(GIT_URL)
                .arg(".")
                .spawn()?
                .wait()?;
        }
        Ok(())
    }
    fn perf_all(&mut self) -> Result<Vec<Stats>> {
        let mut results = vec![];
        for i in self.commits.iter() {
            results.extend(self.perf_commit(i.as_ref())?);
        }
        Ok(results)
    }
    fn perf_commit(&mut self, hash: &str) -> Result<Vec<Stats>> {
        eprintln!("> commit {hash}");
        self.create_command_git_checkout(hash).spawn()?.wait()?;

        let mut results = vec![];
        for file in fs::read_dir("./programs/").unwrap() {
            let file = file?.path();
            if file
                .extension()
                .is_some_and(|x| x.to_string_lossy() == "hvmc")
            {
                results.extend(self.perf_file(&file)?);
            }
        }
        results
            .iter_mut()
            .for_each(|x| x.hash = Some(hash.to_owned()));
        Ok(results)
    }
    fn perf_file(&mut self, file: &Path) -> Result<Vec<Stats>> {
        eprintln!(">> file {file}", file = file.to_string_lossy());
        let mut results = vec![];
        results.extend(self.perf_interpreted(file, false)?);
        results.extend(self.perf_interpreted(file, true)?);
        results.extend(self.with_compiled::<Vec<Stats>>(file, |this, binary| {
            let mut results = vec![];
            results.extend(this.perf_compiled(binary, false)?);
            results.extend(this.perf_compiled(binary, true)?);
            Ok(results)
        })?);
        results
            .iter_mut()
            .for_each(|x| x.file = Some(file.to_string_lossy().into_owned()));
        Ok(results)
    }

    /// Run a function after compiling a HVMC file into a binary
    fn with_compiled<T>(
        &mut self,
        file: &Path,
        f: impl FnOnce(&mut Self, &Path) -> Result<T>,
    ) -> Result<T> {
        let binary = file.with_extension("");

        let file_relative_to_cargo = {
            let mut p = PathBuf::from("..");
            p.push(file);
            p
        };

        if binary.exists() {
            fs::remove_file(&binary)?;
        }

        let mut command = self.create_command_cargo_run();
        command.arg("compile").arg(&file_relative_to_cargo);
        let _ = self.run_and_capture_stdout_err(&mut command)?;

        let result = f(self, &binary);

        if binary.exists() {
            fs::remove_file(&binary)?;
        }

        result
    }
    fn perf_compiled(&mut self, binary: &Path, single: bool) -> Result<Vec<Stats>> {
        eprintln!(">>> mode compiled, -1: {}", single);

        let mut results = if binary.exists() {
            let mut command = Command::new(&binary);
            // if ptr-refactor hasn't been implemented, pass dummy arg
            if !self.is_git_ancestor("9bdbdcbe0816345545a3adf00704f9f4f01dcfe7", "HEAD")? {
                command.arg("_");
            }
            command.arg("-s");
            if single {
                command.arg("-1");
            }

            if self.is_git_ancestor("0ba064c", "HEAD")? {
                command.arg("-m").arg("4G");
            }

            let out = self.run_and_capture_stdout_err(&mut command)?;
            let result = self.parse_output(&out)?;
            eprintln!(">>>> {}", result.show_short());
            vec![result]
        } else {
            eprintln(">>>> compile failed");
            vec![]
        };
        let mode = format!("comp-{}", if single { "singl" } else { "multi" });
        results.iter_mut().for_each(|x| x.mode = Some(mode.clone()));
        Ok(results)
    }
    fn perf_interpreted(&mut self, file: &Path, single: bool) -> Result<Vec<Stats>> {
        eprintln!(">>> mode interpreted, -1: {}", single);
        let file = {
            let mut p = PathBuf::from("..");
            p.push(file);
            p
        };
        let mut command = self.create_command_cargo_run();
        command.arg("run");

        if self.is_git_ancestor("0ba064c", "HEAD")? {
            command.arg("-m").arg("4G");
        }
        command.arg(&file).arg("-s");
        if single {
            command.arg("-1");
        }
        let out = self.run_and_capture_stdout_err(&mut command)?;

        let result = self.parse_output(&out)?;
        eprintln!(">>>> {}", result.show_short());
        let mut results = vec![result];
        let mode = format!("intr-{}", if single { "singl" } else { "multi" });
        results.iter_mut().for_each(|x| x.mode = Some(mode.clone()));
        Ok(results)
    }
    fn parse_output(&mut self, s: &str) -> Result<Stats> {
        Ok(Stats {
            time: self
                .re_time
                .captures(s)
                .map(|x| x.extract::<1>().1[0].to_owned()),
            rwps: self
                .re_rwps
                .captures(s)
                .map(|x| x.extract::<1>().1[0].to_owned()),
            rwts: self
                .re_rwts
                .captures(s)
                .map(|x| x.extract::<1>().1[0].to_owned()),
            ..Default::default()
        })
    }
    fn create_command_git(&mut self) -> Command {
        let mut command = Command::new("git");
        command.arg("-C").arg(&self.git_dir);
        command
    }
    fn create_command_git_checkout(&mut self, hash: &str) -> Command {
        let mut command = self.create_command_git();
        command.arg("checkout").arg(hash);
        command
    }
    fn create_command_git_clone(&mut self, url: &str) -> Command {
        let mut command = self.create_command_git();
        command.arg("clone").arg(url);
        command
    }
    fn create_command_cargo_run(&mut self) -> Command {
        let mut command = Command::new("cargo");

        command
            .current_dir(&self.crate_dir)
            .arg("run")
            .arg("--release")
            .arg("--");
        command
    }
    fn is_git_ancestor(&mut self, ancestor: &str, descendant: &str) -> Result<bool> {
        let mut c = self.create_command_git();
        c.arg("merge-base")
            .arg("--is-ancestor")
            .arg(ancestor)
            .arg(descendant);
        Ok(c.spawn()?.wait()?.success())
    }
    fn run_and_capture_stdout_err(&mut self, command: &mut Command) -> Result<String> {
        let mut cmd = command
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        use wait_timeout::ChildExt;

        let stdout = cmd.stdout.take();
        let stderr = cmd.stderr.take();
        let exit = cmd.wait_timeout(MAX_TIME)?;
        let mut output = String::new();
        if let Some(exit) = exit {
            stdout.unwrap().read_to_string(&mut output)?;
            stderr.unwrap().read_to_string(&mut output)?;
            if !exit.success() {
                eprintln!(">>>> warn: exit status {:?}", exit);
                eprintln!(">>>>> {:?}", output);
            } else {
                eprintln!(">>>> ok");
            }
        } else {
            eprintln!(">>>> warn: timeout");
            cmd.kill()?;
            stdout.unwrap().read_to_string(&mut output)?;
            stderr.unwrap().read_to_string(&mut output)?;
            eprintln!(">>>> killed");
        }
        Ok(output)
    }
}
