use super::*;

const MAX_TIME: Duration = Duration::from_secs(120);

const REV_CLAP_CLI: &str = "0ba064c";
const REV_PTR_REFACTOR: &str = "9bdbdcb";

pub struct Bencher {
  pub core_dir: PathBuf,
  pub bins_dir: PathBuf,

  pub config: BenchConfig,

  pub reporter: Reporter,
}

impl Bencher {
  pub fn init(&mut self) -> Result<()> {
    const GIT_URL: &str = "https://github.com/HigherOrderCO/hvm-core.git";
    if !self.core_dir.exists() {
      fs::create_dir_all(&self.core_dir)?;
    }
    if self.core_dir.read_dir()?.count() == 0 {
      self.git().arg("clone").arg(GIT_URL).arg(".").spawn()?.wait()?;
    }
    if self.config.modes.is_empty() {
      self.config.modes.extend_from_slice(ALL_MODES);
    }
    if self.config.files.is_empty() {
      self.config.files = fs::read_dir("./programs/")
        .unwrap()
        .map(|x| -> Result<_> { Ok(x?.path()) })
        .filter(|x| !x.as_ref().is_ok_and(|x| x.extension() != Some(OsStr::new("hvmc"))))
        .collect::<Result<Vec<PathBuf>>>()?;
    }
    if self.config.revs.is_empty() {
      self.config.revs = String::from_utf8(self.git().arg("tag").output()?.stdout)?
        .split("\n")
        .filter(|x| x.starts_with("compare-"))
        .map(|x| x.to_owned())
        .collect();
      self
        .config
        .revs
        .sort_unstable_by_key(|x| x["compare-".len() ..].split_once('-').unwrap().0.parse::<usize>().unwrap());
    }
    Ok(())
  }

  pub fn bench_all(&self) -> Result<Vec<Datum>> {
    let mut data = vec![];
    let count = self.config.revs.len();
    for (i, rev) in self.config.revs.iter().enumerate() {
      report!(self, "rev {i}/{count}: {rev}"; {
        self.bench_rev(rev.as_ref(), &mut data)?;
      })?;
    }
    Ok(data)
  }

  fn bench_rev(&self, rev: &str, data: &mut Vec<Datum>) -> Result<()> {
    self.git().arg("checkout").arg(rev).output()?;

    let count = self.config.files.len();

    for (i, file) in self.config.files.iter().enumerate() {
      let x = file.display();
      report!(self, "file {i}/{count}: {}", x; {
        self.bench_rev_file(rev, &file, data)?;
      })?;
    }

    Ok(())
  }

  fn bench_rev_file(&self, rev: &str, file: &Path, data: &mut Vec<Datum>) -> Result<()> {
    let hvmc = self.build(rev, file)?;
    for &mode in &self.config.modes {
      let stats = if mode.compiled {
        if let Ok(binary) = self.compile(rev, file, &hvmc) {
          report!(self, "{}", mode; self.bench_compiled(&binary, mode.multi)?).ok()
        } else {
          None
        }
      } else {
        report!(self, "{}", mode; self.bench_interpreted(&hvmc, file, mode.multi)?).ok()
      };
      data.push(Datum {
        rev: rev.to_owned(),
        file: file.file_name().unwrap().to_string_lossy().into_owned(),
        mode,
        stats,
      })
    }
    Ok(())
  }

  fn build(&self, rev: &str, file: &Path) -> Result<PathBuf> {
    let mut binary = self.bins_dir.clone();
    binary.push(rev);
    binary.push("hvmc");

    let mut relative_file = PathBuf::from("..");
    relative_file.push(file);

    if !binary.exists() {
      fs::create_dir_all(binary.parent().unwrap())?;
      report!(self, "building"; {
        self.run_and_capture_stdout_err(Command::new("cargo").current_dir(&self.core_dir).arg("build").arg("--release"))?;
        fs::rename("./hvm-core/target/release/hvmc", &binary)?;
      })?;
    }

    Ok(binary)
  }

  fn compile(&self, rev: &str, file: &Path, hvmc: &Path) -> Result<PathBuf> {
    let mut binary = self.bins_dir.clone();
    binary.push(rev);
    binary.push(file.file_name().unwrap());
    binary.set_extension("");

    let mut relative_file = PathBuf::from("..");
    relative_file.push(file);

    if !binary.exists() {
      fs::create_dir_all(binary.parent().unwrap())?;
      report!(self, "compiling"; {
        self.run_and_capture_stdout_err(&mut Command::new(hvmc).arg("compile").arg(&relative_file))?;
        fs::rename(file.with_extension(""), &binary)?;
      })?;
    }

    Ok(binary)
  }

  fn bench_compiled(&self, binary: &Path, single: bool) -> Result<Stats> {
    let mut command = Command::new(&binary);

    // if ptr-refactor hasn't been implemented, pass dummy arg
    if !self.is_git_ancestor(REV_PTR_REFACTOR)? {
      command.arg("_");
    }

    command.arg("-s");

    if single {
      command.arg("-1");
    }

    if self.is_git_ancestor(REV_CLAP_CLI)? {
      command.arg("-m").arg("4G");
    }

    let out = self.run_and_capture_stdout_err(&mut command)?;
    self.parse_output(&out)
  }

  fn bench_interpreted(&self, hvmc: &Path, file: &Path, multi: bool) -> Result<Stats> {
    let mut command = Command::new(hvmc);
    command.arg("run");

    if self.is_git_ancestor(REV_CLAP_CLI)? {
      command.arg("-m").arg("4G");
    }

    command.arg(&file).arg("-s");

    if !multi {
      command.arg("-1");
    }

    let out = self.run_and_capture_stdout_err(&mut command)?;
    self.parse_output(&out)
  }

  fn parse_output(&self, s: &str) -> Result<Stats> {
    Ok(Stats {
      time: parse_time(regex!(r"TIME *: *(.+)").captures(s).map(|x| x.extract::<1>().1[0]).ok_or(Error::MissingTime)?)?,
      rwts: parse_rwts(regex!(r"RWTS *: *(.+)").captures(s).map(|x| x.extract::<1>().1[0]).ok_or(Error::MissingRwts)?)?,
    })
  }

  fn git(&self) -> Command {
    let mut command = Command::new("git");
    command.current_dir(&self.core_dir);
    command
  }

  fn is_git_ancestor(&self, ancestor: &str) -> Result<bool> {
    Ok(self.git().arg("merge-base").arg("--is-ancestor").arg(ancestor).arg("HEAD").output()?.status.success())
  }

  fn run_and_capture_stdout_err(&self, command: &mut Command) -> Result<String> {
    let mut cmd = command.stderr(Stdio::piped()).stdout(Stdio::piped()).spawn()?;

    use wait_timeout::ChildExt;

    let stdout = cmd.stdout.take();
    let stderr = cmd.stderr.take();
    let exit = cmd.wait_timeout(MAX_TIME)?;
    let mut output = String::new();
    if let Some(exit) = exit {
      stdout.unwrap().read_to_string(&mut output)?;
      stderr.unwrap().read_to_string(&mut output)?;
      exit.exit_ok()?;
    } else {
      self.reporter.message("timeout");
      cmd.kill()?;
      stdout.unwrap().read_to_string(&mut output)?;
      stderr.unwrap().read_to_string(&mut output)?;
    }
    Ok(output)
  }
}

fn parse_time(time: &str) -> Result<f64> {
  let (num, scale) = None
    .or_else(|| time.strip_suffix("µs").map(|x| (x, 1.0e-6)))
    .or_else(|| time.strip_suffix("ms").map(|x| (x, 1.0e-3)))
    .or_else(|| time.strip_suffix("s").map(|x| (x, 1.0)))
    .ok_or(Error::InvalidTime)?;
  Ok(num.trim().parse::<f64>()? * scale)
}

fn parse_rwts(rwts: &str) -> Result<u64> {
  Ok(rwts.replace('_', "").parse()?)
}
