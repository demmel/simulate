use crate::ggez::render::Drawable;
use crate::Statistics;
use ggez::{
  graphics::{self, DrawParam, Image, Rect},
  Context, GameError, GameResult,
};
use plotters::{
  chart::SeriesLabelPosition,
  drawing::{BitMapBackend, IntoDrawingArea},
  prelude::{ChartBuilder, LineSeries, Palette99, Rectangle},
  style::{Color as PlottersColor, IntoFont, Palette},
};
use std::marker::PhantomData;

pub struct StatsChart<'a, TState, TStatistics: Statistics<TState>> {
  max_value: f64,
  min_value: f64,
  stats: &'a Vec<&'a (usize, TStatistics)>,
  _state: PhantomData<TState>,
}

impl<'a, TState, TStatistics: Statistics<TState>> StatsChart<'a, TState, TStatistics> {
  pub fn new(stats: &'a Vec<&'a (usize, TStatistics)>, max_value: f64, min_value: f64) -> Self {
    Self {
      max_value,
      min_value,
      stats,
      _state: PhantomData,
    }
  }
}

impl<'a, TState, TStatistics: Statistics<TState>> Drawable for StatsChart<'a, TState, TStatistics> {
  fn draw(&self, ctx: &mut Context, at: Rect) -> GameResult<()> {
    let Rect { x, y, w, h } = at;
    let mut buffer = vec![255; w as usize * h as usize * 3 /* RGB */];

    {
      let backend = BitMapBackend::with_buffer(&mut buffer, (w as u32, h as u32));
      let root = backend.into_drawing_area();
      root
        .fill(&plotters::prelude::BLACK)
        .or_else(|_| Err(GameError::RenderError(String::from("Could not fill root"))))?;

      let mut cc = ChartBuilder::on(&root)
        .margin(10)
        .caption(
          TStatistics::get_title(),
          ("sans-serif", 30)
            .into_font()
            .color(&plotters::prelude::WHITE),
        )
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_ranged(
          self.stats.first().unwrap().0 as u32..self.stats.last().unwrap().0 as u32,
          self.min_value..self.max_value,
        )
        .or_else(|_| {
          Err(GameError::RenderError(String::from(
            "Could not construct chart",
          )))
        })?;

      cc.configure_mesh()
        .x_label_formatter(&|x| format!("{}", TStatistics::map_tick_unit(*x as usize)))
        .y_label_formatter(&|y| format!("{}", y))
        .x_labels(10)
        .y_labels(10)
        .x_desc(TStatistics::get_tick_unit())
        .y_desc(TStatistics::get_unit())
        .label_style(
          ("sans-serif", 15)
            .into_font()
            .color(&plotters::prelude::WHITE),
        )
        .axis_style(&plotters::prelude::WHITE.to_rgba())
        .axis_desc_style(
          ("sans-serif", 15)
            .into_font()
            .color(&plotters::prelude::WHITE),
        )
        .line_style_1(&plotters::prelude::WHITE.mix(0.5))
        .line_style_2(&plotters::prelude::WHITE.mix(0.25))
        .draw()
        .or_else(|_| {
          Err(GameError::RenderError(String::from(
            "Could not draw chart mesh",
          )))
        })?;

      for (i, name) in TStatistics::get_names().iter().enumerate() {
        cc.draw_series(LineSeries::new(
          self
            .stats
            .iter()
            .map(|(a, b)| (*a as u32, b.get_value(name))),
          &Palette99::pick(i),
        ))
        .or_else(|_| {
          Err(GameError::RenderError(format!(
            "Could not draw '{}' series",
            name
          )))
        })?
        .label(name)
        .legend(move |(x, y)| {
          Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], &Palette99::pick(i))
        });
      }

      cc.configure_series_labels()
        .background_style(&plotters::prelude::BLACK.mix(0.8))
        .border_style(&plotters::prelude::WHITE)
        .label_font(
          ("sans-serif", 15)
            .into_font()
            .color(&plotters::prelude::WHITE),
        )
        .position(SeriesLabelPosition::MiddleLeft)
        .draw()
        .or_else(|_| {
          Err(GameError::RenderError(String::from(
            "Could not draw legend",
          )))
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
