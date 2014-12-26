use xlib::Window;
use std::cmp::min;
use std::num::Float;
use std::fmt;

pub type LayoutBox = ||:'static -> Box<Layout>;

#[deriving(Copy)]
pub struct Rect {
  pub x: u32,
  pub y: u32,
  pub width: u32,
  pub height: u32
}

impl fmt::Show for Rect {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{{ x: {}, y: {}, width: {}, height: {} }}", self.x, self.y, self.width, self.height)
  }
}

pub trait Layout {
  fn name(&self) -> String;
  fn apply(&self, Rect, &Vec<Window>) -> Vec<Rect>;
}

pub struct TallLayout {
  num_masters: uint,
  ratio: f32,
  ratio_increment: f32
}

impl TallLayout {
  pub fn new(num_masters: uint, ratio: f32, ratio_increment: f32) -> TallLayout {
    TallLayout {
      num_masters: num_masters,
      ratio: ratio,
      ratio_increment: ratio_increment
    }
  }
}

impl Layout for TallLayout {
  fn name(&self) -> String {
    String::from_str("Tall")
  }

  fn apply(&self, area: Rect, windows: &Vec<Window>) -> Vec<Rect> {
    Vec::from_fn(windows.len(), |i| {
      if i < self.num_masters {
        let yoff = area.height / min(self.num_masters, windows.len()) as u32;

        Rect{x: area.x, y: area.y + (yoff * i as u32), width: (area.width as f32 * (1.0 - (windows.len() > self.num_masters) as u32 as f32 * (1.0 - self.ratio))).floor() as u32 , height: yoff}
      } else {
        let yoff = area.height / (windows.len() - self.num_masters) as u32;

        Rect{x: area.x + (area.width as f32 * self.ratio).floor() as u32, y: area.y + (yoff * (i - self.num_masters) as u32), width: (area.width as f32 * (1.0 - self.ratio)).floor() as u32 , height: yoff}
      }
    })
  }
}

pub struct BarLayout {
  top: u32,
  bottom: u32,
  layout: Box<Layout + 'static>
}

impl BarLayout {
  pub fn new<T: Layout + 'static>(top: u32, bottom: u32, layout: T) -> BarLayout {
    BarLayout {
      top: top,
      bottom: bottom,
      layout: box layout as Box<Layout>
    }
  }
}

impl Layout for BarLayout {
  fn name(&self) -> String {
    self.layout.name()
  }

  fn apply(&self, area: Rect, windows: &Vec<Window>) -> Vec<Rect> {
    self.layout.apply(Rect{x: area.x, y: area.y + self.top, width: area.width, height: area.height - (self.top + self.bottom)}, windows)
  }
}
