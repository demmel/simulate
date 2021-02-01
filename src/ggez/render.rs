pub(crate) mod icicle_chart;
pub(crate) mod layout;
pub(crate) mod line_chart;
pub(crate) mod simulation;

use ggez::{graphics::Rect, Context, GameResult};

pub trait Drawable {
  fn draw(&self, context: &mut Context, at: Rect) -> GameResult<()>;
}
