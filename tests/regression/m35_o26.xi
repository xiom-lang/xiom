// M35-O26: Result[Float64,Int] -- float Ok, int Err construction and match
fn main() -> Int {
  var a: Result[Float64, Int] = Ok(3.0);
  match a { Ok(_) => {} Err(_) => { return 1; } }
  var b: Result[Float64, Int] = Err(-1);
  match b { Ok(_) => { return 2; } Err(_) => {} }
  var c = Ok(0.0);
  match c { Ok(_) => {} Err(_) => { return 3; } }
  var d = Err(99);
  match d { Ok(_) => { return 4; } Err(e) => { if e != 99 { return 5; } } }
  return 0;
}
