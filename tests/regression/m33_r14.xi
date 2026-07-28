fn unwrap_opt(o: Option[Int], default: Int) -> Int {
  match o { Some(v) => v, None => default }
}
fn unwrap_res(r: Result[Int, Str], fallback: Int) -> Int {
  match r { Ok(v) => v, Err(_) => fallback }
}
fn main() -> Int {
  var a: Option[Int] = Some(10);
  var b: Option[Int] = None;
  var r1 = unwrap_opt(a, 0) + unwrap_opt(b, 5) * 3;
  if r1 != 25 { return 1; }
  var c: Result[Int, Str] = Err("fail");
  var d: Result[Int, Str] = Ok(7);
  var r2 = unwrap_res(c, 100) + unwrap_res(d, 0);
  if r2 != 107 { return 2; }
  return 0;
}
