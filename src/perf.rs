#[derive(Clone)]
pub struct Perf(Vec<flame::Span>);

impl Perf {
  pub fn get() -> Self {
    Self(flame::spans())
  }

  pub fn get_perf(
    &self,
    name: &str,
    delim: &str,
  ) -> Option<std::time::Duration> {
    get_perf_from_spans(&self.0, name, delim)
  }
}

pub(crate) fn clear() {
  flame::clear();
}

pub fn span_of<T, F>(name: &'static str, f: F) -> T
where
  F: FnOnce() -> T,
{
  flame::span_of(name, f)
}

fn get_perf_from_span(
  span: &flame::Span,
  name: &str,
  delim: &str,
) -> Option<std::time::Duration> {
  if let Some(rest) = name.strip_prefix(span.name.as_ref()) {
    if rest.is_empty() {
      Some(std::time::Duration::from_nanos(span.delta))
    } else if let Some(rest) = rest.strip_prefix(delim) {
      get_perf_from_spans(&span.children, rest, delim)
    } else {
      None
    }
  } else {
    None
  }
}

fn get_perf_from_spans(
  spans: &[flame::Span],
  name: &str,
  delim: &str,
) -> Option<std::time::Duration> {
  let mut overall_duration = None;
  for span in spans {
    if let Some(duration) = get_perf_from_span(span, name, delim) {
      overall_duration =
        overall_duration.map(|d| d + duration).or(Some(duration));
    }
  }
  overall_duration
}
