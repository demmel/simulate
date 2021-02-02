use indexmap::IndexMap;
use std::{borrow::Cow, fmt::Display, iter::FromIterator, time::Duration};

#[derive(Clone)]
pub struct Perf(Vec<flame::Span>);

impl Perf {
  pub fn get() -> Self {
    Self(flame::threads().into_iter().flat_map(|t| t.spans).collect())
  }

  pub fn get_perf(
    &self,
    name: &str,
    delim: &str,
  ) -> Option<std::time::Duration> {
    get_perf_from_spans(&self.0, name, delim)
  }

  pub fn folded(&self) -> FoldedSpans {
    fold_spans(&self.0)
  }
}

impl Display for Perf {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    fold_spans(&self.0).fmt(f)
  }
}

impl FromIterator<Perf> for Perf {
  fn from_iter<T: IntoIterator<Item = Perf>>(iter: T) -> Self {
    Perf(iter.into_iter().flat_map(|p| p.0).collect())
  }
}

impl<'a> FromIterator<&'a Perf> for Perf {
  fn from_iter<T: IntoIterator<Item = &'a Perf>>(iter: T) -> Self {
    Perf(iter.into_iter().flat_map(|p| p.0.clone()).collect())
  }
}

pub fn clear() {
  flame::clear();
}

pub fn span_of<T, F>(name: &'static str, f: F) -> T
where
  F: FnOnce() -> T,
{
  flame::span_of(name, f)
}

pub fn start(name: &'static str) {
  flame::start(name)
}

pub fn end(name: &'static str) {
  flame::end(name);
}

pub struct FoldedSpans(IndexMap<Cow<'static, str>, FoldedSpan>);

impl FoldedSpans {
  pub fn duration(&self) -> Duration {
    self.0.values().map(|s| s.duration).sum()
  }

  pub fn spans(
    &self,
  ) -> impl Iterator<Item = (&Cow<'static, str>, &FoldedSpan)> {
    self.0.iter()
  }

  pub fn get(&self, name: &[&str]) -> Option<&FoldedSpans> {
    let mut base = self;
    for &name in name {
      if let Some(span) = base.0.get(name) {
        base = &span.children;
      } else {
        return None;
      }
    }
    Some(base)
  }
}

impl Display for FoldedSpans {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let indent = f.width().unwrap_or(0);
    let longest = self.0.keys().map(|s| s.len()).max().unwrap_or(0);
    let longest_duration = self
      .0
      .values()
      .map(|s| format!("{:?}", s.duration).len())
      .max()
      .unwrap_or(0);
    for (name, span) in self.0.iter() {
      writeln!(
        f,
        "{}{:width$} : {:duration_width$} ({:?})",
        "  ".repeat(indent),
        name,
        format!("{:?}", span.duration),
        span.duration / span.num_folded as u32,
        width = longest,
        duration_width = longest_duration,
      )?;
      write!(f, "{:indent$}", span.children, indent = indent + 1)?;
    }
    Ok(())
  }
}

pub struct FoldedSpan {
  duration: Duration,
  num_folded: usize,
  children: FoldedSpans,
}

impl FoldedSpan {
  pub fn duration(&self) -> Duration {
    self.duration
  }

  pub fn average_duration(&self) -> Duration {
    self.duration / self.num_folded as u32
  }

  pub fn children(&self) -> &FoldedSpans {
    &self.children
  }
}

fn fold_spans(spans: &[flame::Span]) -> FoldedSpans {
  let mut folded_spans = FoldedSpans(IndexMap::new());
  for span in spans {
    fold_span_into(&mut folded_spans, span);
  }
  folded_spans
}

fn fold_span_into(folded: &mut FoldedSpans, span: &flame::Span) {
  folded
    .0
    .entry(span.name.to_owned())
    .and_modify(|folded| {
      folded.duration += Duration::from_nanos(span.delta);
      folded.num_folded += 1;
      for span in &span.children {
        fold_span_into(&mut folded.children, span);
      }
    })
    .or_insert_with(|| FoldedSpan {
      duration: Duration::from_nanos(span.delta),
      num_folded: 1,
      children: fold_spans(&span.children),
    });
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
