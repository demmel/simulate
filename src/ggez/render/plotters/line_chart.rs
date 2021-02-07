use crate::stats::{SimStats, Statistics, StatisticsGroup};

use plotters::{
  chart::SeriesLabelPosition,
  coord::Shift,
  prelude::{
    BitMapBackend, ChartBuilder, DrawingArea, DrawingAreaErrorKind,
    DrawingBackend, LineSeries, Palette99, Rectangle,
  },
  style::{Color as PlottersColor, IntoFont, Palette},
};

use super::PlottersDrawableAdapter;

pub(crate) struct StatsCharts<'a, TState, TStatistics: Statistics<TState>> {
  stats: &'a SimStats<TState, TStatistics>,
}

impl<'a, TState, TStatistics: Statistics<TState>>
  StatsCharts<'a, TState, TStatistics>
{
  pub fn new(stats: &'a SimStats<TState, TStatistics>) -> Self {
    Self { stats }
  }
}

impl<'a, TState, TStatistics: Statistics<TState>> PlottersDrawableAdapter
  for StatsCharts<'a, TState, TStatistics>
{
  fn draw(
    &self,
    drawing_area: &DrawingArea<BitMapBackend, Shift>,
  ) -> Result<
    (),
    DrawingAreaErrorKind<<BitMapBackend as DrawingBackend>::ErrorType>,
  > {
    drawing_area.fill(&plotters::prelude::BLACK)?;
    let (xs, ys) = drawing_area.get_pixel_range();
    let (w, h) = (xs.end - xs.start, ys.end - ys.start);

    let groups = TStatistics::get_groups();
    let n_groups = groups.len();
    let (r, c) = (1..=n_groups)
      .map(|r| (r, n_groups / r + if n_groups % r != 0 { 1 } else { 0 }))
      .filter(|(r, c)| {
        let (w, h) = (w as f64 / *c as f64, h as f64 / *r as f64);
        println!("{} {} {}", w, h, w / h);
        true
      })
      .min_by(|(r1, c1), (r2, c2)| {
        let (w1, h1) = (w as f64 / *c1 as f64, h as f64 / *r1 as f64);
        let (w2, h2) = (w as f64 / *c2 as f64, h as f64 / *r2 as f64);
        let a1 = w1 / h1;
        let a2 = w2 / h2;
        a1.ln().abs().partial_cmp(&a2.ln().abs()).unwrap()
      })
      .unwrap();

    let cells = drawing_area.split_evenly((r, c));

    for (i, group) in groups.iter().enumerate() {
      StatsChart::new(
        group,
        &self.stats.statistics,
        self.stats.min_values[i],
        self.stats.max_values[i],
      )
      .draw(&cells[i])?;
    }

    Ok(())
  }
}

pub(crate) struct StatsChart<'a, TState, TStatistics: Statistics<TState>> {
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

impl<'a, TState, TStatistics: Statistics<TState>> PlottersDrawableAdapter
  for StatsChart<'a, TState, TStatistics>
{
  fn draw(
    &self,
    drawing_area: &DrawingArea<BitMapBackend, Shift>,
  ) -> Result<
    (),
    DrawingAreaErrorKind<<BitMapBackend as DrawingBackend>::ErrorType>,
  > {
    let (xs, ys) = drawing_area.get_pixel_range();
    let (_, h) = ((xs.end - xs.start) as f64, (ys.end - ys.start) as f64);

    let &Self {
      min_value,
      mut max_value,
      ..
    } = self;

    if max_value <= min_value {
      max_value = min_value + f64::EPSILON;
    }

    {
      let mut cc = ChartBuilder::on(&drawing_area)
        .margin(10)
        .caption(
          &self.group.title,
          ("sans-serif", h / 15.0)
            .into_font()
            .color(&plotters::prelude::WHITE),
        )
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(
          self.stats.first().unwrap().0 as u32
            ..self.stats.last().unwrap().0 as u32,
          min_value..max_value,
        )?;

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
        .bold_line_style(&plotters::prelude::WHITE.mix(0.5))
        .light_line_style(&plotters::prelude::WHITE.mix(0.25))
        .draw()?;

      for (i, name) in self.group.names.iter().enumerate() {
        cc.draw_series(LineSeries::new(
          self
            .stats
            .iter()
            .map(|(a, b)| (*a as u32, b.get_value(name.clone()))),
          &Palette99::pick(i),
        ))?
        .label(format!("{}", name))
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
        .draw()?;
    }

    Ok(())
  }
}
