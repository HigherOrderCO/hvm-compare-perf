use std::str::FromStr;

use super::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Mode {
  pub compiled: bool,
  pub multi: bool,
}

impl Display for Mode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}-{}", if self.compiled { "comp" } else { "intr" }, if self.multi { "multi" } else { "singl" })
  }
}

pub const ALL_MODES: &[Mode] = &[
  Mode { compiled: false, multi: false },
  Mode { compiled: false, multi: true },
  Mode { compiled: true, multi: false },
  Mode { compiled: true, multi: true },
];

impl ValueEnum for Mode {
  fn value_variants<'a>() -> &'a [Self] {
    ALL_MODES
  }

  fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
    Some(PossibleValue::new(format!("{}", self)))
  }
}

impl FromStr for Mode {
  type Err = anyhow::Error;
  fn from_str(s: &str) -> Result<Self> {
    let (compiled, multi) = s.split_once('-').ok_or(Error::InvalidMode)?;
    Ok(Mode {
      compiled: match compiled {
        "comp" => true,
        "intr" => false,
        _ => Err(Error::InvalidMode)?,
      },
      multi: match multi {
        "multi" => true,
        "singl" => false,
        _ => Err(Error::InvalidMode)?,
      },
    })
  }
}

#[derive(Debug)]
pub struct Datum {
  pub rev: String,
  pub file: String,
  pub mode: Mode,
  pub stats: Option<Stats>,
}

#[derive(Debug)]
pub struct Stats {
  pub time: f64,
  pub rwts: u64,
}
