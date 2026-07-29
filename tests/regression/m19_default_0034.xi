module regression.m19_default_0034

interface Divisible {
  fn div2(&self) -> Result[Int, Str] {
    var v = value();
    if v % 2 == 0 { return Ok(v / 2); }
    return Err("odd");
  }
  fn value(&self) -> Int;
}

type Odd = { num: Int; }

fn Odd.value(&self) -> Int { return num; }

fn main() -> Int {
  var o: Odd = Odd{ num: 7 };
  var r = o.div2();
  match r {
    Ok(_) => return 1,
    Err(e) => if e == "odd" { return 0; }
  }
  return 2;
}
