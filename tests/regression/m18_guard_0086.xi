module regression.m18_guard_0086

fn main() -> Int {
  var opt: Option[Option[Int]] = Some(Some(7));
  match opt {
    x if x is Some(inner) && inner is Some(v) && v > 0 => { return 0; }
    _ => { return 1; }
  }
}
