use super::Layout;
use ggez::graphics::Rect;

enum FlexKind {
  Row,
  Column,
}

pub struct Flex<T> {
  items: Vec<FlexItem<T>>,
  kind: FlexKind,
}

impl<T> Flex<T> {
  pub fn row(items: Vec<FlexItem<T>>) -> Layout<T> {
    Layout::Flex(Self {
      items,
      kind: FlexKind::Row,
    })
  }

  pub fn column(items: Vec<FlexItem<T>>) -> Layout<T> {
    Layout::Flex(Self {
      items,
      kind: FlexKind::Column,
    })
  }

  pub fn try_visit<F, E>(&self, bounds: Rect, f: &mut F) -> Result<(), E>
  where
    F: FnMut(&T, Rect) -> Result<(), E>,
  {
    let total_weight: f32 = self.items.iter().map(|i| i.weight).sum();

    let mut start = 0.0;
    for item in self.items.iter() {
      let relative_weight = item.weight / total_weight;
      let size = relative_weight
        * match self.kind {
          FlexKind::Row => bounds.w,
          FlexKind::Column => bounds.h,
        };
      let item_bounds = match self.kind {
        FlexKind::Row => Rect {
          x: bounds.x + start,
          y: bounds.y,
          w: size,
          h: bounds.h,
        },
        FlexKind::Column => Rect {
          x: bounds.x,
          y: bounds.y + start,
          w: bounds.w,
          h: size,
        },
      };
      item.item.try_visit(item_bounds, f)?;
      start += size;
    }

    Ok(())
  }
}

pub struct FlexItem<T> {
  pub weight: f32,
  pub item: Layout<T>,
}
