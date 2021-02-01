use ggez::{
  graphics::{self, DrawParam, Image, Rect},
  Context, GameError, GameResult,
};
use plotters::{
  coord::Shift,
  prelude::{
    BitMapBackend, DrawingArea, DrawingAreaErrorKind, DrawingBackend,
    IntoDrawingArea,
  },
};

use super::Drawable;

pub(crate) mod icicle_chart;
pub(crate) mod line_chart;

pub trait PlottersDrawableAdapter {
  fn draw(
    &self,
    drawing_area: &DrawingArea<BitMapBackend, Shift>,
  ) -> Result<
    (),
    DrawingAreaErrorKind<<BitMapBackend as DrawingBackend>::ErrorType>,
  >;
}

impl<T> Drawable for T
where
  T: PlottersDrawableAdapter,
{
  fn draw(&self, ctx: &mut Context, at: Rect) -> GameResult<()> {
    let Rect { x, y, w, h } = at;
    let mut buffer = vec![255; w as usize * h as usize * 3 /* RGB */];

    {
      let backend =
        BitMapBackend::with_buffer(&mut buffer, (w as u32, h as u32));
      let root = backend.into_drawing_area();
      self.draw(&root).map_err(|_| {
        GameError::RenderError(String::from("Problem rendering chart"))
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
