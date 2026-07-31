module regression.m19_default_0065

interface Area {
  fn area(&self) -> Float64 { return width() * height(); }
  fn width(&self) -> Float64;
  fn height(&self) -> Float64;
}

type Rect = { w: Float64; h: Float64; }

fn Rect.area(self) -> Float64 { return self.width() * self.height(); }


fn Rect.width(&self) -> Float64 { return w; }

fn Rect.height(&self) -> Float64 { return h; }

fn main() -> Int {
  var r: Rect = Rect{ w: 5.0, h: 3.0 };
  if r.width() == 5.0 && r.height() == 3.0 && r.area() == 15.0 { return 0; }
  return 1;
}
