#![feature(exit_status_error, absolute_path, lazy_cell)]

use std::{
  cell::Cell,
  ffi::OsStr,
  fmt::{Display, Write},
  io::{Read, Write as _},
  path::{Path, PathBuf},
  process::{Command, Stdio},
  sync::LazyLock,
  time::{Duration, Instant},
};

use fs::File;
use fs_err as fs;

use anyhow::Result;
use chrono::prelude::*;
use clap::{builder::PossibleValue, Parser, Subcommand, ValueEnum};
use regex::Regex;
use thiserror::Error;

mod bencher;
mod csv;
mod pretty;
mod reporter;
mod types;
mod util;

use bencher::*;
use reporter::*;
use types::*;
use util::*;

use crate::pretty::pretty_print_data;

#[derive(Parser)]
struct Cli {
  #[command(subcommand)]
  command: CliCommand,
}

#[derive(Subcommand, Debug)]
enum CliCommand {
  Bench {
    #[command(flatten)]
    config: BenchConfig,
  },
  Show {
    csv: String,
  },
}

#[derive(Parser, Debug)]
struct BenchConfig {
  #[arg(short = 'r', long = "rev")]
  revs: Vec<String>,
  #[arg(short = 'f', long = "file")]
  files: Vec<PathBuf>,
  #[arg(short = 'm', long = "mode")]
  modes: Vec<Mode>,
}

pub fn main() -> Result<()> {
  match Cli::parse().command {
    CliCommand::Bench { config } => {
      let mut state =
        Bencher { core_dir: "./hvm-core".into(), bins_dir: "./bins".into(), config, reporter: Default::default() };

      state.init()?;

      let data = state.bench_all()?;

      fs::create_dir_all("./out")?;
      let mut file = File::create(format!("./out/{}.csv", Utc::now().format("%Y-%m-%d-%H-%M-%S")))?;
      use std::io::Write;
      write!(&mut file, "{}", Datum::to_csv(&data))?;

      std::io::stdout().write_all(pretty_print_data(data).as_slice()).unwrap();
    }
    CliCommand::Show { csv } => {
      let data = Datum::from_csv(&String::from_utf8(fs::read(csv)?)?)?;
      std::io::stdout().write_all(pretty_print_data(data).as_slice()).unwrap();
    }
  }

  Ok(())
}
