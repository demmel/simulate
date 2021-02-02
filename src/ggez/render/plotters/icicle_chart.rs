use std::time::Duration;

use plotters::{
  coord::Shift,
  prelude::{
    BitMapBackend, DrawingArea, DrawingAreaErrorKind, DrawingBackend, Rectangle,
  },
  style::{Color, IntoFont, Palette, Palette99, BLACK, WHITE},
};

use crate::perf::{FoldedSpan, FoldedSpans};

use super::PlottersDrawableAdapter;

const LAYER_HEIGHT: i32 = 20;

pub(crate) struct PerfChart<'a> {
  perf: &'a FoldedSpans,
}

impl<'a> PerfChart<'a> {
  pub fn new(perf: &'a FoldedSpans) -> Self {
    Self { perf }
  }
}

impl<'a> PlottersDrawableAdapter for PerfChart<'a> {
  fn draw(
    &self,
    da: &DrawingArea<BitMapBackend, Shift>,
  ) -> Result<
    (),
    DrawingAreaErrorKind<<BitMapBackend as DrawingBackend>::ErrorType>,
  > {
    da.fill(&plotters::prelude::BLACK)?;

    let (xs, ys) = da.get_pixel_range();
    let (_, h) = (xs.end - xs.start, ys.end - ys.start);
    let font = ("sans-serif", h / 15).into_font();
    let da = da.titled("Performance", font.color(&WHITE))?;
    let font = ("sans-serif", h / 20).into_font();
    let da = da.titled(
      &format!(
        "{}",
        HumanReadableDuration(&self.perf.sum_of_average_durations())
      ),
      font.color(&WHITE),
    )?;

    let da = da.margin(0, 15, 15, 15);

    draw_spans(&da, &self.perf, 0)?;

    Ok(())
  }
}

fn draw_spans<DB: DrawingBackend>(
  da: &DrawingArea<DB, Shift>,
  spans: &FoldedSpans,
  depth: usize,
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
  let total = spans.sum_of_average_durations().as_secs_f64();

  let mut next_da = da.clone();
  for (name, span) in spans.spans() {
    let split = next_da.split_horizontally(
      da.relative_to_width(span.average_duration().as_secs_f64() / total)
        as i32,
    );
    next_da = split.1;
    draw_span(&split.0, name, span, depth)?;
  }

  Ok(())
}

fn draw_span<DB: DrawingBackend>(
  da: &DrawingArea<DB, Shift>,
  name: &str,
  span: &FoldedSpan,
  depth: usize,
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
  let (current, rest) = da.split_vertically(LAYER_HEIGHT);

  draw_span_header(&current, name, depth)?;

  if span.children().spans().next().is_none() {
    return Ok(());
  } else if span.children().sum_of_average_durations() < span.average_duration()
  {
    let percent = span.children().sum_of_average_durations().as_secs_f64()
      / span.average_duration().as_secs_f64();
    let break_at = rest.relative_to_width(percent) as i32;
    let (children, missing) = rest.split_horizontally(break_at);
    draw_spans(&children, span.children(), depth + 1)?;
    draw_span_header(
      &missing.split_vertically(LAYER_HEIGHT).0,
      "Missing",
      depth + 1,
    )?;
  } else {
    draw_spans(&rest, span.children(), depth + 1)?;
  }

  Ok(())
}

fn draw_span_header<DB: DrawingBackend>(
  da: &DrawingArea<DB, Shift>,
  name: &str,
  depth: usize,
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
  let (xs, ys) = da.get_pixel_range();
  let (w, h) = (xs.end - xs.start, ys.end - ys.start);

  if w < 4 {
    return Ok(());
  }

  da.draw(&Rectangle::new(
    [(1, 1), (w - 1, h - 1)],
    Palette99::pick(depth).filled(),
  ))?;

  da.draw(&Rectangle::new([(0, 0), (w, h)], BLACK.stroke_width(2)))?;

  let font = ("sans-serif", h / 2).into_font();

  let mut trunc_name = name;
  let (mut tw, mut th) = da.estimate_text_size(name, &font)?;
  while tw as i32 > w && !trunc_name.is_empty() {
    trunc_name = &trunc_name[..(trunc_name.len() - 1)];
    let (ntw, nth) = da.estimate_text_size(trunc_name, &font)?;
    tw = ntw;
    th = nth;
  }
  if !trunc_name.is_empty() {
    da.draw_text(
      trunc_name,
      &font.color(&BLACK),
      (w / 2 - (tw / 2) as i32, h / 2 - (th / 2) as i32),
    )?;
  }

  Ok(())
}

pub struct HumanReadableDuration<'a>(&'a Duration);

impl<'a> std::fmt::Display for HumanReadableDuration<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let duration = self.0;

    if duration.as_nanos() < 10000 {
      write!(f, "{} ns", duration.as_nanos())
    } else if duration.as_micros() < 10000 {
      write!(f, "{} µs", duration.as_micros())
    } else if duration.as_millis() < 10000 {
      write!(f, "{} ms", duration.as_millis())
    } else {
      write!(f, "{} s", duration.as_secs_f64())
    }
  }
}
