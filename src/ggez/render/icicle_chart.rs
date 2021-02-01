use ggez::{
  graphics::{self, DrawParam, Image, Rect},
  Context, GameError, GameResult,
};
use plotters::{
  coord::Shift,
  prelude::{
    BitMapBackend, DrawingArea, DrawingAreaErrorKind, DrawingBackend,
    IntoDrawingArea, Rectangle,
  },
  style::{Color, IntoFont, Palette, Palette99, BLACK},
};

use crate::perf::{FoldedSpan, FoldedSpans, Perf};

use super::Drawable;

pub struct PerfChart<'a> {
  perf: &'a Perf,
}

impl<'a> PerfChart<'a> {
  pub fn new(perf: &'a Perf) -> Self {
    Self { perf }
  }
}

impl<'a> Drawable for PerfChart<'a> {
  fn draw(&self, ctx: &mut Context, at: Rect) -> GameResult<()> {
    let Rect { x, y, w, h } = at;
    let mut buffer = vec![255; w as usize * h as usize * 3 /* RGB */];

    {
      let backend =
        BitMapBackend::with_buffer(&mut buffer, (w as u32, h as u32));
      let root = backend.into_drawing_area();
      root.fill(&plotters::prelude::BLACK).map_err(|_| {
        GameError::RenderError(String::from("Could not fill root"))
      })?;

      draw_spans(&root, &self.perf.folded(), 0).map_err(|_| {
        GameError::RenderError(String::from("Could not draw icicles"))
      })?;
    }

    let image = Image::from_rgba8(
      ctx,
      w as u16,
      h as u16,
      &buffer.chunks(3).enumerate().fold(
        vec![255; w as usize * h as usize * 4 /* RGBA */],
        |mut buf, (ci, cur)| {
          for i in 0..3 {
            buf[4 * ci + i] = cur[i];
          }
          buf
        },
      ),
    )?;
    graphics::draw(ctx, &image, DrawParam::default().dest([x, y]))?;

    Ok(())
  }
}

fn draw_spans<DB: DrawingBackend>(
  da: &DrawingArea<DB, Shift>,
  spans: &FoldedSpans,
  depth: usize,
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
  let total = spans.duration().as_secs_f64();

  let mut next_da = da.clone();
  for (name, span) in spans.spans() {
    let split = next_da.split_horizontally(
      da.relative_to_width(span.duration().as_secs_f64() / total) as i32,
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
  let (current, rest) = da.split_vertically(30);

  draw_span_header(&current, name, depth)?;

  if span.children().spans().next().is_none() {
    return Ok(());
  } else if span.children().duration() < span.duration() {
    let percent =
      span.children().duration().as_secs_f64() / span.duration().as_secs_f64();
    let break_at = rest.relative_to_width(percent) as i32;
    let (children, missing) = rest.split_horizontally(break_at);
    draw_spans(&children, span.children(), depth + 1)?;
    draw_span_header(&missing.split_vertically(30).0, "Missing", depth + 1)?;
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

  let font = ("sans-serif", h / 4).into_font();

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
