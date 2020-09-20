use crate::ggez::render::{chart::StatsChart, simulation::StateRenderer, Drawable as MyDrawable};
use crate::{Simulation, Statistics, StatisticsTrackingSimulator};
use ggez::{
  event::{EventHandler, MouseButton},
  graphics::{self, Drawable, Rect},
  mint::Point2,
  timer, Context, GameResult,
};
use std::time::{Duration, Instant};

pub struct App<TSimulation, TStatistics>
where
  TSimulation: Simulation,
  TStatistics: Statistics<TSimulation::TState>,
  TSimulation::TState: Drawable,
{
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
}

impl<TSimulation, TStatistics> App<TSimulation, TStatistics>
where
  TSimulation: Simulation,
  TStatistics: Statistics<TSimulation::TState>,
  TSimulation::TState: Drawable,
{
  pub fn new(
    ctx: &mut Context,
    simulator: StatisticsTrackingSimulator<TSimulation, TStatistics>,
  ) -> Self {
    let drawable_size = graphics::drawable_size(&ctx);

    Self {
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
    }
  }
}

impl<TSimulation, TStatistics> EventHandler for App<TSimulation, TStatistics>
where
  TSimulation: Simulation,
  TStatistics: Statistics<TSimulation::TState>,
  TSimulation::TState: Drawable,
{
  fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
    let update_start = Instant::now();
    if let Some((w, h)) = self.new_size {
      graphics::set_screen_coordinates(ctx, Rect { x: 0., y: 0., w, h })?;
      self.drawable_size = [w, h];
      self.new_size = None;
    }

    let target_fps = 60.0;

    let mut time_available = Duration::from_secs_f32(1.0 / target_fps)
      .checked_sub(self.draw_time.unwrap_or(Duration::new(0, 0)))
      .unwrap_or_else(|| Duration::new(0, 1));

    while self.ticks as f32 / timer::time_since_start(ctx).as_secs_f32() < self.tick_rate
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

    self.update_time = Some(Instant::now() - update_start);

    Ok(())
  }

  fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
    self.mouse_pos = Point2::from([x, y]);
    if self.mouse_down {
      self.camera_position[0] -= dx;
      self.camera_position[1] -= dy;
    }
  }

  fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
    match button {
      MouseButton::Left => {
        self.mouse_down = true;
      }
      _ => {}
    }
  }

  fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
    match button {
      MouseButton::Left => {
        self.mouse_down = false;
      }
      _ => {}
    }
  }

  fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
    self.zoom_level += 0.05 * y;
    self.zoom_level = self.zoom_level.max(0.05);
  }

  fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
    let draw_start = Instant::now();

    graphics::clear(ctx, graphics::BLACK);
    let (w, h) = graphics::drawable_size(ctx);
    StateRenderer::new(self.simulator.state())
      .zoom_level(self.zoom_level)
      .camera_position(self.camera_position)
      .draw(
        ctx,
        Rect {
          x: w / 2.0,
          y: 0.0,
          w: w / 2.0,
          h,
        },
      )?;

    let stats: Vec<_> = self.simulator.statistics().collect();
    StatsChart::new(&stats).draw(
      ctx,
      Rect {
        x: 0.0,
        y: 0.0,
        w: w / 2.0,
        h,
      },
    )?;

    graphics::present(ctx)?;

    self.draw_time = Some(Instant::now() - draw_start);

    println!(
      "Update: {:?} Draw: {:?} Delta: {:?} FPS: {:?} TPS: {:?}",
      self.update_time,
      self.draw_time,
      timer::delta(ctx),
      timer::fps(ctx),
      self.ticks as f32 / timer::time_since_start(ctx).as_secs_f32(),
    );

    Ok(())
  }

  fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) {
    self.new_size = Some((width, height));
  }
}
