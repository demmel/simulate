pub mod flex;

pub use self::flex::*;
use super::Drawable;
use ggez::{graphics::Rect, Context, GameResult};
use std::{collections::HashMap, hash::Hash};

pub enum Layout<T> {
  Flex(Flex<T>),
  Layers(Vec<Layout<T>>),
  Leaf(T),
}

impl<T> Layout<T> {
  pub fn try_visit<F, E>(&self, bounds: Rect, f: &mut F) -> Result<(), E>
  where
    F: FnMut(&T, Rect) -> Result<(), E>,
  {
    match self {
      Layout::Flex(flex) => flex.try_visit(bounds, f),
      Layout::Layers(layers) => {
        for layer in layers.iter() {
          layer.try_visit(bounds, f)?;
        }
        Ok(())
      }
      Layout::Leaf(leaf) => f(leaf, bounds),
    }
  }
}

impl<'a> Drawable for Layout<&'a dyn Drawable> {
  fn draw(&self, context: &mut Context, at: Rect) -> GameResult<()> {
    self.try_visit(at, &mut |item, bounds| item.draw(context, bounds))
  }
}

impl<T> Layout<T>
where
  T: Clone + Hash + Eq,
{
  pub fn layout(&self, bounds: Rect) -> HashMap<T, Rect> {
    let mut layout = HashMap::new();
    self
      .try_visit::<_, ()>(bounds, &mut |item, bounds| {
        layout.insert(item.clone(), bounds);
        Ok(())
      })
      .expect("Should never fail");
    layout
  }
}

impl<T> Layout<T>
where
  T: Eq,
{
  pub fn get(&self, item: &T, bounds: Rect) -> Option<Rect> {
    self
      .try_visit::<_, Rect>(bounds, &mut |visited, bounds| {
        if visited == item {
          Err(bounds)
        } else {
          Ok(())
        }
      })
      .err()
  }
}
