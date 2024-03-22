use super::*;

const HEADER: &str = "rev,file,mode,time,rwts\n";

impl Datum {
  pub fn to_csv(entries: &[Datum]) -> String {
    let mut csv = HEADER.to_owned();
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
  pub fn from_csv(csv: &str) -> Result<Vec<Datum>> {
    let csv = csv.strip_prefix(HEADER).ok_or(Error::InvalidCsv)?;
    csv
      .lines()
      .map(|line| {
        let mut els = line.split(',');
        let rev = els.next().ok_or(Error::InvalidCsv)?.to_owned();
        let file = els.next().ok_or(Error::InvalidCsv)?.to_owned();
        let mode = els.next().ok_or(Error::InvalidCsv)?.parse()?;
        let stats = match els.next().ok_or(Error::InvalidCsv)? {
          "" => {
            if els.next() != Some("") {
              Err(Error::InvalidCsv)?
            }
            None
          }
          time => Some(Stats { time: time.parse()?, rwts: els.next().ok_or(Error::InvalidCsv)?.parse()? }),
        };
        if els.next() != None {
          Err(Error::InvalidCsv)?
        }
        Ok(Datum { rev, file, mode, stats })
      })
      .collect()
  }
}
