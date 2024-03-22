use super::*;

#[derive(Default)]
pub struct Reporter {
  indent: Cell<usize>,
  needs_newline: Cell<bool>,
}

impl Reporter {
  fn start_line(&self) {
    if self.needs_newline.get() {
      eprintln!();
      self.needs_newline.set(false);
    }
    for _ in 0 .. self.indent.get() {
      eprint!("  ");
    }
  }
  pub fn message(&self, line: impl Display) {
    self.start_line();
    eprintln!("{}", line);
  }
  pub fn report<T>(&self, line: impl Display, f: impl FnOnce() -> Result<T>) -> Result<T> {
    self.start_line();
    eprint!("{}...", line);
    self.indent.set(self.indent.get() + 1);
    self.needs_newline.set(true);
    let start = Instant::now();
    let out = f();
    let elapsed = start.elapsed();
    self.indent.set(self.indent.get() - 1);
    if !self.needs_newline.get() {
      self.start_line();
      eprint!("...");
    }
    if let Err(e) = &out {
      eprintln!(" {}", e);
    } else {
      eprintln!(" {:.3?}", elapsed);
    }
    self.needs_newline.set(false);
    out
  }
}
