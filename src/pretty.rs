use crate::Datum;
use termcolor::{Color, ColorSpec, WriteColor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Align {
  Left,
  Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Style {
  Neutral,
  Header,
  Bucket0,
  Bucket1,
  Bucket2,
  Bucket3,
  Bucket4,
}

#[derive(Debug)]
struct Cell {
  align: Align,
  style: Style,
  content: String,
  min_width: usize,
}

#[derive(Debug)]
enum Row {
  Separator(char),
  Cells(Vec<Cell>),
}

const REV_WIDTH: usize = 12;

pub fn pretty_print_data(data: Vec<Datum>, buf: &mut impl WriteColor) {
  let revs = collect_uniq(data.iter().map(|x| &x.rev));
  let files = collect_uniq(data.iter().map(|x| &x.file));
  let modes = collect_uniq(data.iter().map(|x| &x.mode));

  let mut table = Vec::new();

  table.push(Row::Cells(
    [Cell { align: Align::Left, style: Style::Header, content: "file".to_owned(), min_width: 0 }, Cell {
      align: Align::Left,
      style: Style::Header,
      content: "mode".to_owned(),
      min_width: 0,
    }]
    .into_iter()
    .chain(revs.iter().map(|x| Cell {
      align: Align::Left,
      style: Style::Header,
      content: x.strip_prefix("compare-").unwrap_or(x)[0 .. REV_WIDTH].to_owned(),
      min_width: REV_WIDTH,
    }))
    .collect(),
  ));

  for (i, file) in files.iter().enumerate() {
    table.push(Row::Separator(if i == 0 { '=' } else { '-' }));
    for (i, mode) in modes.iter().enumerate() {
      let mut cells = revs
        .iter()
        .map(|rev| data.iter().find(|x| &x.rev == rev && &x.file == file && &x.mode == mode).unwrap())
        .map(|x| x.stats.as_ref().map(|x| x.time))
        .map(|x| (x, Style::Bucket2))
        .collect::<Vec<_>>();
      let mut sorted_cells = cells.iter_mut().filter_map(|x| Some((x.0?, &mut x.1))).collect::<Vec<_>>();
      sorted_cells.sort_by(|a, b| f64::total_cmp(&a.0, &b.0));
      if sorted_cells.len() > 1 {
        let third = ((sorted_cells.len() - 2) / 3) + 1;
        *sorted_cells[0].1 = Style::Bucket0;
        for i in 1 .. third {
          *sorted_cells[i].1 = Style::Bucket1;
        }
        for i in sorted_cells.len() - third .. sorted_cells.len() - 1 {
          *sorted_cells[i].1 = Style::Bucket3;
        }
        *sorted_cells.last_mut().unwrap().1 = Style::Bucket4;
      }
      table.push(Row::Cells(
        [
          Cell {
            align: Align::Left,
            style: Style::Neutral,
            content: if i == 0 { file.strip_suffix(".hvmc").unwrap_or(file).to_string() } else { String::new() },
            min_width: 0,
          },
          Cell { align: Align::Left, style: Style::Neutral, content: mode.to_string(), min_width: 0 },
        ]
        .into_iter()
        .chain(cells.iter().map(|cell| Cell {
          align: Align::Right,
          style: cell.1,
          content: cell.0.map(|x| format!("{:.3} s", x)).unwrap_or_default(),
          min_width: 0,
        }))
        .collect(),
      ));
    }
  }

  print_table(table, buf);
}

fn print_table(table: Vec<Row>, buf: &mut impl WriteColor) {
  let rows = table.iter().filter_map(|x| match x {
    Row::Cells(x) => Some(x),
    _ => None,
  });
  let max_lengths = rows
    .clone()
    .next()
    .unwrap()
    .iter()
    .enumerate()
    .map(|(i, _)| rows.clone().map(|x| x[i].content.len().max(x[i].min_width)).max().unwrap())
    .collect::<Vec<_>>();
  let gap_size = 2;
  let width = max_lengths.iter().copied().sum::<usize>() + (max_lengths.len() - 1) * gap_size;
  for row in table {
    match row {
      Row::Separator(char) => {
        for _ in 0 .. width {
          write!(buf, "{}", char).unwrap();
        }
      }
      Row::Cells(cells) => {
        for (i, cell) in cells.iter().enumerate() {
          if i != 0 {
            write!(buf, "  ").unwrap();
          }
          let pad = max_lengths[i] - cell.content.len();
          if cell.align == Align::Right {
            for _ in 0 .. pad {
              write!(buf, " ").unwrap();
            }
          }
          let mut spec = ColorSpec::new();
          buf
            .set_color(match cell.style {
              Style::Neutral => &spec,
              Style::Header => spec.set_bold(true),
              Style::Bucket0 => spec.set_fg(Some(Color::Green)).set_bold(true),
              Style::Bucket1 => spec.set_fg(Some(Color::Green)),
              Style::Bucket2 => &spec,
              Style::Bucket3 => spec.set_fg(Some(Color::Red)),
              Style::Bucket4 => spec.set_fg(Some(Color::Red)).set_bold(true),
            })
            .unwrap();
          write!(buf, "{}", &cell.content).unwrap();
          buf.set_color(&ColorSpec::new()).unwrap();
          if cell.align == Align::Left {
            for _ in 0 .. pad {
              write!(buf, " ").unwrap();
            }
          }
        }
      }
    }
    write!(buf, "\n").unwrap();
  }
}

fn collect_uniq<'a, T: 'a + Eq + Clone>(iter: impl IntoIterator<Item = &'a T>) -> Vec<T> {
  let mut vec = Vec::new();
  for item in iter {
    if !vec.contains(item) {
      vec.push(item.clone());
    }
  }
  vec
}
