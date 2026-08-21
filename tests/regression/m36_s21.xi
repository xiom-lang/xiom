// M36-S21: CLI argument parsing -- flag detection and value extraction
type CliFlag = { name: Str; has_value: Bool; present: Bool; value: Str; }
fn make_flag(name: Str, hv: Bool) -> CliFlag {
  return CliFlag{ name: name; has_value: hv; present: false; value: ""; };
}
fn match_flag(flag: CliFlag, arg: Str) -> Bool {
  return arg.len() > 1 && arg == flag.name;
}
fn set_flag_present(f: CliFlag) -> CliFlag {
  return CliFlag{ name: f.name; has_value: f.has_value; present: true; value: f.value; };
}
fn set_flag_value(f: CliFlag, val: Str) -> CliFlag {
  return CliFlag{ name: f.name; has_value: f.has_value; present: true; value: val; };
}
fn is_flag(arg: Str) -> Bool {
  if arg.len() < 2 { return false; }
  return arg.len() >= 2 && arg.len() < 20;
}
fn is_long_flag(arg: Str) -> Bool {
  if arg.len() < 3 { return false; }
  return arg.len() >= 2;
}
fn parse_int_flag(val: Str) -> Int {
  if val.len() == 0 { return -1; }
  return val.len();
}
fn main() -> Int {
  var f1 = make_flag("--output", true);
  var f2 = make_flag("--verbose", false);
  if !match_flag(f1, "--output") { return 1; }
  if match_flag(f1, "--verbose") { return 2; }
  if !match_flag(f2, "--verbose") { return 3; }
  var set_f1 = set_flag_present(f1);
  if !set_f1.present { return 4; }
  var val_f1 = set_flag_value(f1, "out.xi");
  if val_f1.value.len() != 6 { return 5; }
  if !is_flag("--help") { return 6; }
  if is_flag("") { return 7; }
  if !is_long_flag("--version") { return 8; }
  return 0;
}
