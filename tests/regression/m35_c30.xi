// M35-C30: all-in-one — combined control flow patterns in single test
enum Kind { Small, Medium, Large }
type Item = { value: Int; kind: Kind; }
fn process_item(it: Item) -> Int {
  var v: Int = it.value;
  var k: Kind = it.kind;
  var result: Int = 0;
  match k {
    Small => { if v < 10 { result = 1; } else { result = 2; } }
    Medium => {
      var i: Int = 0;
      while i < v { result = result + i; i = i + 1; if result > 100 { break; } }
      if i == 0 { result = 3; }
    }
    Large => {
      if v > 50 { return 99; }
      var acc: Int = 1;
      var j: Int = 1;
      while j <= v { acc = acc * j; j = j + 1; if acc > 1000 { acc = 1000; break; } }
      result = acc;
    }
  }
  return result;
}
fn classify(n: Int) -> Int {
  var r: Int = match n {
    0 => 0,
    1 => 10,
    2 => 20,
    _ => if n > 5 { 100 } else { 50 },
  };
  return r;
}
fn main() -> Int {
  var i1 = Item{ value: 5; kind: Kind.Small; };
  var i2 = Item{ value: 15; kind: Kind.Small; };
  var i3 = Item{ value: 0; kind: Kind.Medium; };
  var i4 = Item{ value: 5; kind: Kind.Medium; };
  var i5 = Item{ value: 60; kind: Kind.Large; };
  var i6 = Item{ value: 4; kind: Kind.Large; };
  if process_item(i1) != 1 { return 1; }
  if process_item(i2) != 2 { return 2; }
  if process_item(i3) != 3 { return 3; }
  if process_item(i4) != 10 { return 4; }
  if process_item(i5) != 99 { return 5; }
  if process_item(i6) != 24 { return 6; }
  if classify(0) != 0 { return 7; }
  if classify(1) != 10 { return 8; }
  if classify(2) != 20 { return 9; }
  if classify(3) != 50 { return 10; }
  if classify(7) != 100 { return 11; }
  var opt: Option[Int] = Some(42);
  match opt { Some(v) => { if v != 42 { return 12; } } None => { return 13; } }
  return 0;
}
