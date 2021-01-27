use crate::ggez::render::Drawable;
use crate::stats::SimStats;
use crate::stats::Statistics;
use crate::stats::StatisticsGroup;
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

pub struct StatsCharts<'a, TState, TStatistics: Statistics<TState>> {
  stats: &'a SimStats<TState, TStatistics>,
}

impl<'a, TState, TStatistics: Statistics<TState>>
  StatsCharts<'a, TState, TStatistics>
{
  pub fn new(stats: &'a SimStats<TState, TStatistics>) -> Self {
    Self { stats }
  }
}

impl<'a, TState, TStatistics: Statistics<TState>> Drawable
  for StatsCharts<'a, TState, TStatistics>
{
  fn draw(&self, ctx: &mut Context, at: Rect) -> GameResult<()> {
    let Rect { x, y, w, h } = at;

    let groups = TStatistics::get_groups();
    let grid_size = (groups.len() as f64).sqrt().ceil() as usize;
    let sx = w / grid_size as f32;
    let sy = h / grid_size as f32;

    for (i, group) in groups.iter().enumerate() {
      let r = i / grid_size;
      let c = i % grid_size;
      StatsChart::new(
        group,
        &self.stats.statistics,
        self.stats.min_values[i],
        self.stats.max_values[i],
      )
      .draw(
        ctx,
        Rect {
          x: x + c as f32 * sx,
          y: y + r as f32 * sy,
          w: sx,
          h: sy,
        },
      )?;
    }

    Ok(())
  }
}

pub struct StatsChart<'a, TState, TStatistics: Statistics<TState>> {
  max_value: f64,
  min_value: f64,
  group: &'a StatisticsGroup<TState, TStatistics>,
  stats: &'a [(usize, TStatistics)],
}

impl<'a, TState, TStatistics: Statistics<TState>>
  StatsChart<'a, TState, TStatistics>
{
  pub fn new(
    group: &'a StatisticsGroup<TState, TStatistics>,
    stats: &'a [(usize, TStatistics)],
    min_value: f64,
    max_value: f64,
  ) -> Self {
    Self {
      min_value,
      max_value,
      group,
      stats,
    }
  }
}

impl<'a, TState, TStatistics: Statistics<TState>> Drawable
  for StatsChart<'a, TState, TStatistics>
{
  fn draw(&self, ctx: &mut Context, at: Rect) -> GameResult<()> {
    let Rect { x, y, w, h } = at;
    let mut buffer = vec![255; w as usize * h as usize * 3 /* RGB */];

    let &Self {
      min_value,
      mut max_value,
      ..
    } = self;

    if max_value <= min_value {
      max_value = min_value + f64::EPSILON;
    }

    {
      let backend =
        BitMapBackend::with_buffer(&mut buffer, (w as u32, h as u32));
      let root = backend.into_drawing_area();
      root.fill(&plotters::prelude::BLACK).map_err(|_| {
        GameError::RenderError(String::from("Could not fill root"))
      })?;

      let mut cc = ChartBuilder::on(&root)
        .margin(10)
        .caption(
          &self.group.title,
          ("sans-serif", h / 15.0)
            .into_font()
            .color(&plotters::prelude::WHITE),
        )
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_ranged(
          self.stats.first().unwrap().0 as u32
            ..self.stats.last().unwrap().0 as u32,
          min_value..max_value,
        )
        .map_err(|_| {
          GameError::RenderError(String::from("Could not construct chart"))
        })?;

      cc.configure_mesh()
        .x_label_formatter(&|x| {
          format!("{}", TStatistics::map_tick_unit(*x as usize))
        })
        .y_label_formatter(&|y| {
          let (y, u) = match y {
            y if *y >= 1_000_000_000.0 => (y / 1_000_000_000.0, " B"),
            y if *y >= 1_000_000.0 => (y / 1_000_000.0, " M"),
            y if *y >= 1_000.0 => (y / 1_000.0, " K"),
            y => (*y, ""),
          };
          format!("{:.2}{}", y, u)
        })
        .x_labels(10)
        .y_labels(10)
        .x_desc(TStatistics::get_tick_unit())
        .y_desc(&self.group.unit)
        .label_style(
          ("sans-serif", h / 30.0)
            .into_font()
            .color(&plotters::prelude::WHITE),
        )
        .axis_style(&plotters::prelude::WHITE.to_rgba())
        .axis_desc_style(
          ("sans-serif", h / 30.0)
            .into_font()
            .color(&plotters::prelude::WHITE),
        )
        .line_style_1(&plotters::prelude::WHITE.mix(0.5))
        .line_style_2(&plotters::prelude::WHITE.mix(0.25))
        .draw()
        .map_err(|_| {
          GameError::RenderError(String::from("Could not draw chart mesh"))
        })?;

      for (i, name) in self.group.names.iter().enumerate() {
        cc.draw_series(LineSeries::new(
          self
            .stats
            .iter()
            .map(|(a, b)| (*a as u32, b.get_value(name.clone()))),
          &Palette99::pick(i),
        ))
        .map_err(|_| {
          GameError::RenderError(format!(
            "Could not draw '{}' series",
            <TStatistics::TStatID as Into<String>>::into(name.clone())
          ))
        })?
        .label(name.clone())
        .legend(move |(x, y)| {
          Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], &Palette99::pick(i))
        });
      }

      cc.configure_series_labels()
        .background_style(&plotters::prelude::BLACK.mix(0.8))
        .border_style(&plotters::prelude::WHITE)
        .label_font(
          ("sans-serif", h / 30.0)
            .into_font()
            .color(&plotters::prelude::WHITE),
        )
        .position(SeriesLabelPosition::MiddleLeft)
        .draw()
        .map_err(|_| {
          GameError::RenderError(String::from("Could not draw legend"))
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
