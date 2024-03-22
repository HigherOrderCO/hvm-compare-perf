#![feature(exit_status_error, absolute_path, lazy_cell)]

use std::{
  cell::Cell,
  ffi::OsStr,
  fmt::{Display, Write},
  io::Read,
  path::{Path, PathBuf},
  process::{Command, Stdio},
  sync::LazyLock,
  time::{Duration, Instant},
};

use fs::File;
use fs_err as fs;

use anyhow::Result;
use chrono::prelude::*;
use clap::{builder::PossibleValue, Parser, ValueEnum};
use regex::Regex;
use thiserror::Error;

mod bencher;
mod reporter;
mod types;
mod util;

use bencher::*;
use reporter::*;
use types::*;
use util::*;

#[derive(Parser, Debug)]
struct Config {
  #[arg(short = 'r', long = "rev")]
  revs: Vec<String>,
  #[arg(short = 'f', long = "file")]
  files: Vec<PathBuf>,
  #[arg(short = 'm', long = "mode")]
  modes: Vec<Mode>,
}

pub fn main() -> Result<()> {
  let mut state = Bencher {
    core_dir: "./hvm-core".into(),
    bins_dir: "./bins".into(),
    config: Config::parse(),
    reporter: Default::default(),
  };

  state.init()?;

  let output = state.bench_all()?;

  fs::create_dir_all("./out")?;
  let mut file = File::create(format!("./out/{}.csv", Utc::now().format("%Y-%m-%d-%H-%M-%S")))?;
  use std::io::Write;
  write!(&mut file, "{}", Datum::to_csv(&output))?;

  Ok(())
}

impl Datum {
  fn to_csv(entries: &[Datum]) -> String {
    let mut csv = "rev,file,mode,time,rwts\n".to_owned();
    for entry in entries {
      write!(csv, "{},{},{},", entry.rev, entry.file, entry.mode).unwrap();
      match &entry.stats {
        Some(stats) => write!(csv, "{},{}", stats.time, stats.rwts).unwrap(),
        None => csv.push(','),
      }
      csv.push('\n');
    }
    csv
  }
}
