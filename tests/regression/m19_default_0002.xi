module regression.m19_default_0002

interface QnA {
  fn answer(&self) -> Int { return 42; }
  fn question(&self) -> Str;
}

type Thinker = { query: Str; }

fn Thinker.question(&self) -> Str { return query; }

fn main() -> Int {
  var t: Thinker = Thinker{ query: "life" };
  if t.question() == "life" && t.answer() == 42 { return 0; }
  return 1;
}
