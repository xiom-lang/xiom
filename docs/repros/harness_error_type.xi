use repro_error_type;

fn main() -> Int {
  var e = make_error(42);
  if e.code != 42 { return 1; }
  if e.message != "err" { return 2; }
  if error_code(&e) != 42 { return 3; }
  return 0;
}
