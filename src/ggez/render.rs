pub(crate) mod layout;
pub(crate) mod plotters;
pub(crate) mod simulation;

use ggez::{graphics::Rect, Context, GameResult};

pub trait Drawable {
  fn draw(&self, context: &mut Context, at: Rect) -> GameResult<()>;
}
