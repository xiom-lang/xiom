module regression.m19_default_0068

interface TryGet {
  fn try_get(&self) -> Option[Str] {
    var s = name();
    if s.len() > 0 { return Some(s); }
    return None;
  }
  fn name(&self) -> Str;
}

type Empty = { val: Str; }

fn Empty.name(&self) -> Str { return val; }

fn main() -> Int {
  var e: Empty = Empty{ val: "" };
  var result = e.try_get();
  match result {
    Some(_) => return 1,
    None => return 0
  }
}
