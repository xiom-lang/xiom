module regression.m19_default_0020

interface Fallible {
  fn try_get(&self) -> Result[Int, Str] {
    var v = value();
    if v > 0 { return Ok(v); }
    return Err("negative");
  }
  fn value(&self) -> Int;
}

type Source = { val: Int; }

fn Source.value(&self) -> Int { return val; }

fn main() -> Int {
  var good: Source = Source{ val: 10 };
  var bad: Source = Source{ val: -5 };
  var g = good.try_get();
  var b = bad.try_get();
  match g {
    Ok(n) => if n != 10 { return 1; },
    Err(_) => return 2
  }
  match b {
    Ok(_) => return 3,
    Err(e) => if e != "negative" { return 4; }
  }
  return 0;
}
