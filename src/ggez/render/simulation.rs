use crate::ggez::{render::Drawable, StateRenderer};
use ggez::graphics::{DrawParam, Rect};

pub struct InternalStateRenderer<'a, TState>
where
  TState: StateRenderer,
{
  assets: &'a TState::TAssets,
  state: &'a TState,
  zoom_level: f32,
  camera_position: [f32; 2],
}

impl<'a, TState> InternalStateRenderer<'a, TState>
where
  TState: StateRenderer,
{
  pub fn new(state: &'a TState, assets: &'a TState::TAssets) -> Self {
    Self {
      assets,
      state,
      zoom_level: 1.0,
      camera_position: [0.0, 0.0],
    }
  }

  pub fn zoom_level(mut self, zoom_level: f32) -> Self {
    self.zoom_level = zoom_level;
    self
  }

  pub fn camera_position(mut self, camera_position: [f32; 2]) -> Self {
    self.camera_position = camera_position;
    self
  }
}

impl<'a, TState> Drawable for InternalStateRenderer<'a, TState>
where
  TState: StateRenderer,
{
  fn draw(
    &self,
    ctx: &mut ggez::Context,
    at: ggez::graphics::Rect,
  ) -> std::result::Result<(), ggez::GameError> {
    let Rect { w, h, .. } = self.state.dimensions(ctx).unwrap_or(Rect {
      x: 0.0,
      y: 0.0,
      w: 0.0,
      h: 0.0,
    });

    let sx = at.w / w;
    let sy = at.h / h;
    let px_per_m = sx.min(sy);
    let zoom = px_per_m * self.zoom_level;

    let camera = DrawParam::default()
      .dest([
        at.x + at.w / 2.0 - self.camera_position[0],
        at.y + at.h / 2.0 - self.camera_position[1],
      ])
      .scale([zoom, zoom]);

    self.state.draw(ctx, self.assets, camera)?;

    Ok(())
  }
}
