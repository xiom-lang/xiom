// M34-Q14: Contract with string length -- chained requires on string properties
fn str_nonempty(s: Str) -> Bool {
  return s != "";
}
fn str_len(s: Str) -> Int
  requires: str_nonempty(s)
  ensures: result > 0
{
  return s.len();
}
fn str_concat_check(a: Str, b: Str) -> Int
  requires: str_nonempty(a)
  requires: str_nonempty(b)
  ensures: result > str_len(a)
{
  var len_a = str_len(a);
  var len_b = str_len(b);
  return len_a + len_b;
}
fn first_char_val(s: Str) -> Int
  requires: str_nonempty(s)
{
  return str_len(s);
}
fn main() -> Int {
  var r1 = str_len("hello");
  var r2 = str_concat_check("abc", "def");
  var r3 = first_char_val("XIOM");
  if r1 == 5 && r2 == 6 && r3 == 4 { return 0; }
  return 1;
}
