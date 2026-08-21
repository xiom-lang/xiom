// M35-O30: nested Option/Option and Result/Result -- double nesting via custom types
enum DoubleOpt { First(v: Int), Second, Neither }
fn classify_opt(o: Option[Int]) -> DoubleOpt {
  match o { Some(v) => { if v > 0 { return DoubleOpt.First(v); } return DoubleOpt.Second; } None => DoubleOpt.Neither }
}
enum DoubleRes { Win(v: Int), Lose(reason: Str), Fatal }
fn classify_res(r: Result[Int, Str]) -> DoubleRes {
  match r { Ok(v) => { if v >= 0 { return DoubleRes.Win(v); } return DoubleRes.Fatal; } Err(e) => DoubleRes.Lose(e) }
}
fn main() -> Int {
  match classify_opt(Some(42)) { DoubleOpt.First(v) => { if v != 42 { return 1; } } _ => { return 2; } }
  match classify_opt(None) { DoubleOpt.Neither => {} _ => { return 3; } }
  match classify_res(Ok(99)) { DoubleRes.Win(v) => { if v != 99 { return 4; } } _ => { return 5; } }
  match classify_res(Err("fail")) { DoubleRes.Lose(e) => { if e != "fail" { return 6; } } _ => { return 7; } }
  return 0;
}
