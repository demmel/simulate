use super::{
  render::{
    layout::{Flex, FlexItem, Layout},
    plotters::{icicle_chart::PerfChart, line_chart::StatsCharts},
    simulation::InternalStateRenderer,
  },
  StateRenderer,
};
use crate::{
  perf::{self, Perf},
  stats::{Statistics, StatisticsTrackingSimulator},
  Simulation,
};
use ggez::{
  event::{EventHandler, MouseButton},
  graphics::{self, Rect},
  mint::Point2,
  timer, Context, GameResult,
};
use std::{
  collections::VecDeque,
  time::{Duration, Instant},
};

pub struct App<TSimulation, TStatistics>
where
  TSimulation: Simulation,
  TStatistics: Statistics<TSimulation::TState>,
  TSimulation::TState: StateRenderer,
{
  assets: <TSimulation::TState as StateRenderer>::TAssets,
  drawable_size: [f32; 2],
  mouse_pos: Point2<f32>,
  mouse_down: bool,
  new_size: Option<(f32, f32)>,
  update_time: Option<Duration>,
  draw_time: Option<Duration>,
  tick_rate: f32,
  camera_position: [f32; 2],
  ticks: u32,
  simulator: StatisticsTrackingSimulator<TSimulation, TStatistics>,
  zoom_level: f32,
  layout: Layout<AppSection>,
  perf: VecDeque<Perf>,
}

impl<TSimulation, TStatistics> App<TSimulation, TStatistics>
where
  TSimulation: Simulation,
  TStatistics: Statistics<TSimulation::TState>,
  TSimulation::TState: StateRenderer,
{
  pub fn new(
    ctx: &mut Context,
    simulator: StatisticsTrackingSimulator<TSimulation, TStatistics>,
  ) -> Result<Self, ggez::GameError> {
    let drawable_size = graphics::drawable_size(&ctx);

    Ok(Self {
      assets: simulator.state().load_assets(ctx)?,
      mouse_pos: ggez::input::mouse::position(ctx),
      mouse_down: ggez::input::mouse::button_pressed(ctx, MouseButton::Left),
      drawable_size: [drawable_size.0, drawable_size.1],
      new_size: None,
      update_time: None,
      draw_time: None,
      tick_rate: 5000.0,
      camera_position: [0.0, 0.0],
      ticks: 0,
      simulator,
      zoom_level: 1.0,
      layout: Layout::Layers(vec![
        Flex::row(vec![
          FlexItem {
            weight: 1.0,
            item: Layout::Leaf(AppSection::None),
          },
          FlexItem {
            weight: 1.0,
            item: Layout::Leaf(AppSection::Simulation),
          },
        ]),
        Flex::row(vec![
          FlexItem {
            weight: 1.0,
            item: Flex::column(vec![
              FlexItem {
                weight: 3.0,
                item: Layout::Leaf(AppSection::Stats),
              },
              FlexItem {
                weight: 1.0,
                item: Layout::Leaf(AppSection::Perf),
              },
            ]),
          },
          FlexItem {
            weight: 1.0,
            item: Layout::Leaf(AppSection::None),
          },
        ]),
      ]),
      perf: VecDeque::new(),
    })
  }
}

impl<TSimulation, TStatistics> EventHandler for App<TSimulation, TStatistics>
where
  TSimulation: Simulation,
  TStatistics: Statistics<TSimulation::TState>,
  TSimulation::TState: StateRenderer,
{
  fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
    // Perf graph is always one frame behind
    if self.perf.len() > 60 {
      self.perf.pop_front();
    }
    self.perf.push_back(Perf::get());
    perf::clear();

    perf::span_of("Update", || {
      let update_start = Instant::now();
      if let Some((w, h)) = self.new_size {
        graphics::set_screen_coordinates(ctx, Rect { x: 0., y: 0., w, h })?;
        self.drawable_size = [w, h];
        self.new_size = None;
      }

      let target_fps = 60.0;

      let mut time_available = Duration::from_secs_f32(1.0 / target_fps)
        .checked_sub(self.draw_time.unwrap_or_else(|| Duration::new(0, 0)))
        .unwrap_or_else(|| Duration::new(0, 1));

      while self.ticks as f32 / timer::time_since_start(ctx).as_secs_f32()
        < self.tick_rate
        && time_available.as_secs_f32() > 0.0
      {
        let tick_start = Instant::now();
        self.simulator.tick();
        let tick_stop = Instant::now();
        let tick_duration = tick_stop - tick_start;
        self.ticks += 1;
        time_available = time_available
          .checked_sub(tick_duration)
          .unwrap_or_else(|| Duration::new(0, 0));
      }

      self
        .simulator
        .state()
        .update_assets(ctx, &mut self.assets)?;

      self.update_time = Some(Instant::now() - update_start);

      Ok(())
    })
  }

  fn mouse_motion_event(
    &mut self,
    _ctx: &mut Context,
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
  ) {
    self.mouse_pos = Point2::from([x, y]);
    if self.mouse_down {
      self.camera_position[0] -= dx / 1.75 * self.zoom_level.powf(1.0 / 3.0);
      self.camera_position[1] -= dy / 1.75 * self.zoom_level.powf(1.0 / 3.0);
    }
  }

  fn mouse_button_down_event(
    &mut self,
    _ctx: &mut Context,
    button: MouseButton,
    _x: f32,
    _y: f32,
  ) {
    if let MouseButton::Left = button {
      self.mouse_down = true;
    }
  }

  fn mouse_button_up_event(
    &mut self,
    _ctx: &mut Context,
    button: MouseButton,
    _x: f32,
    _y: f32,
  ) {
    if let MouseButton::Left = button {
      self.mouse_down = false;
    }
  }

  fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
    let sim_rect = self
      .layout
      .get(
        &AppSection::Simulation,
        Rect {
          x: 0.0,
          y: 0.0,
          w: self.drawable_size[0],
          h: self.drawable_size[1],
        },
      )
      .unwrap();

    let d_zoom = 0.05 * y * self.zoom_level;
    let o_zoom = self.zoom_level;
    self.zoom_level = (o_zoom + d_zoom).max(0.05);

    let mouse_pos =
      nalgebra::Point2::from([self.mouse_pos.x, self.mouse_pos.y]);
    let center = nalgebra::Point2::from([
      sim_rect.x + sim_rect.w / 2.0,
      sim_rect.y + sim_rect.h / 2.0,
    ]);
    let camera_pos = nalgebra::Vector2::from(self.camera_position);

    let shift = (mouse_pos - center) + camera_pos;

    let zoom_multi = self.zoom_level / o_zoom;

    self.camera_position[0] += shift.x * (zoom_multi - 1.0);
    self.camera_position[1] += shift.y * (zoom_multi - 1.0);
  }

  fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
    perf::span_of("Draw", || {
      let draw_start = Instant::now();

      graphics::clear(ctx, graphics::BLACK);
      let (w, h) = graphics::drawable_size(ctx);

      use super::render::Drawable;

      self.layout.try_visit(
        Rect {
          x: 0.0,
          y: 0.0,
          w,
          h,
        },
        &mut |section, bounds| match section {
          AppSection::None => Ok(()),
          AppSection::Perf => perf::span_of("Perf", || {
            PerfChart::new(&self.perf.iter().collect::<Perf>().folded())
              .draw(ctx, bounds)
          }),
          AppSection::Simulation => perf::span_of("Simulation", || {
            InternalStateRenderer::new(self.simulator.state(), &self.assets)
              .zoom_level(self.zoom_level)
              .camera_position(self.camera_position)
              .draw(ctx, bounds)
          }),
          AppSection::Stats => perf::span_of("Stats", || {
            StatsCharts::new(&self.simulator.stats).draw(ctx, bounds)
          }),
        },
      )?;

      graphics::present(ctx)?;

      self.draw_time = Some(Instant::now() - draw_start);

      Ok(())
    })
  }

  fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) {
    self.new_size = Some((width, height));
  }
}

#[derive(Clone, Hash, PartialEq, Eq)]
enum AppSection {
  None,
  Perf,
  Simulation,
  Stats,
}
