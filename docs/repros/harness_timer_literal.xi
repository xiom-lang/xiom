use repro_timer_literal;

fn main() -> Int {
  var t = make_timer(42);
  var r = read_fields(&t);
  if r != 0 { return r; }
  return 0;
}
