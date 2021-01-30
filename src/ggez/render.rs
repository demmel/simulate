pub mod line_chart;
pub mod simulation;

use ggez::{graphics::Rect, Context, GameResult};

pub trait Drawable {
  fn draw(&self, context: &mut Context, at: Rect) -> GameResult<()>;
}
