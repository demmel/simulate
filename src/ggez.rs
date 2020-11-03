use ggez::graphics::Rect;
use ggez::Context;

pub mod app;
mod render;

pub trait StateRenderer {
  type TAssets;

  fn load_assets(&self, ctx: &mut Context) -> Result<Self::TAssets, ggez::GameError>;

  fn update_assets(
    &self,
    _ctx: &mut Context,
    _assets: &mut Self::TAssets,
  ) -> Result<(), ggez::GameError> {
    Ok(())
  }

  fn draw(&self, ctx: &mut ggez::Context, assets: &Self::TAssets) -> Result<(), ggez::GameError>;

  fn dimensions(&self, _: &mut ggez::Context) -> Option<Rect>;
}
