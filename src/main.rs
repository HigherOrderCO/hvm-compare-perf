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
use clap::{builder::PossibleValue, Parser, Subcommand, ValueEnum};
use regex::Regex;
use termcolor::{ColorChoice, StandardStream};
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
    #[arg(long = "core", default_value = "./hvm-core")]
    core_dir: PathBuf,
    #[arg(long = "out", default_value = "./out")]
    out_dir: PathBuf,
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
    CliCommand::Bench { core_dir, out_dir, config } => {
      let bins_dir = out_dir.join("bins");
      let mut state = Bencher { core_dir, bins_dir, config, reporter: Default::default() };

      state.init()?;

      let data = state.bench_all()?;

      fs::create_dir_all("./out")?;
      let mut file = File::create(format!("./out/{}.csv", Utc::now().format("%Y-%m-%d-%H-%M-%S")))?;
      use std::io::Write;
      write!(&mut file, "{}", Datum::to_csv(&data))?;

      eprintln!();

      pretty_print_data(data, &mut StandardStream::stdout(ColorChoice::Auto));
    }
    CliCommand::Show { csv } => {
      let data = Datum::from_csv(&String::from_utf8(fs::read(csv)?)?)?;
      pretty_print_data(data, &mut StandardStream::stdout(ColorChoice::Auto));
    }
  }

  Ok(())
}
