// m396 positive lock: everything the SPEC allows stays legal.
//   var rebind + field store, shadowing, `var (a, b)`, params, for
//   variables, and `&mut self` receiver stores.
type Point = { x: Int; y: Int; }

fn Point.bump(&mut self) {
  self.x = self.x + 1;
}

fn change(mut n: Int) -> Int {
  n = n + 1;
  return n;
}

fn main() -> Int {
  var p = Point{ x: 0; y: 0; };
  p.x = 1;
  p.bump();
  if p.x != 2 { return 1; }
  p = Point{ x: 5; y: 0; };
  if p.x != 5 { return 2; }

  let s = 1;
  let s = 2;
  if s != 2 { return 3; }

  var (a, b) = (1, 2);
  a = 10;
  if a != 10 { return 4; }

  var items: Vec[Int] = Vec[Int].new();
  items.push(1);
  items.push(2);
  var total = 0;
  for n in items {
    n = n + 1;
    total = total + n;
  }
  if total != 5 { return 5; }

  if change(3) != 4 { return 6; }
  return 0;
}
