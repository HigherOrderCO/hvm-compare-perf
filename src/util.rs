use super::*;

#[macro_export]
macro_rules! report {
  ( $self:ident, $($args:expr),* $(,)? ; $expr:expr ) => {
    $self.reporter.report(format_args!($($args),*), || Ok($expr))
  };
}

#[macro_export]
macro_rules! regex {
  ($source:literal) => {{
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new($source).unwrap());
    &*RE
  }};
}

#[derive(Error, Debug)]
pub enum Error {
  #[error("missing time in hvmc output")]
  MissingTime,
  #[error("invalid time in hvmc output")]
  InvalidTime,
  #[error("missing rwts in hvmc output")]
  MissingRwts,
}
