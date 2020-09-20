use crate::ggez::render::Drawable;
use ggez::graphics::{self, DrawParam, Drawable as GGezDrawable, Rect};

pub struct StateRenderer<'a, TState>
where
  TState: GGezDrawable,
{
  state: &'a TState,
  zoom_level: f32,
  camera_position: [f32; 2],
}

impl<'a, TState> StateRenderer<'a, TState>
where
  TState: GGezDrawable,
{
  pub fn new(state: &'a TState) -> Self {
    Self {
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

impl<'a, TState> Drawable for StateRenderer<'a, TState>
where
  TState: GGezDrawable,
{
  fn draw(
    &self,
    ctx: &mut ggez::Context,
    at: ggez::graphics::Rect,
  ) -> std::result::Result<(), ggez::GameError> {
    let Rect { w, h, .. } = self.state.dimensions(ctx).unwrap_or_else(|| Rect {
      x: 0.0,
      y: 0.0,
      w: 0.0,
      h: 0.0,
    });

    let sx = at.w / w;
    let sy = at.h / h;
    let px_per_m = sx.min(sy);
    let zoom = px_per_m * self.zoom_level;

    // Camera Drawing
    graphics::push_transform(
      ctx,
      Some(
        DrawParam::default()
          .dest([
            at.x + at.w / 2.0 - self.camera_position[0],
            at.y + at.h / 2.0 - self.camera_position[1],
          ])
          .scale([zoom, zoom])
          .to_matrix(),
      ),
    );
    graphics::apply_transformations(ctx)?;

    self.state.draw(ctx, DrawParam::default())?;

    graphics::pop_transform(ctx);
    graphics::apply_transformations(ctx)
  }
}
