module m73_ptr_replace_str
// R19 regression: generic deref-store through `*mut T` used
// `trim_end_matches('*')`, which turned `i8**` (`*mut Str`) into `i8` and
// emitted `store i8 %ptr, i8**` (clang: '%tmp defined with type ptr but
// expected i8') in ptr.replace_Str, reached via mem.replace[Str].
// The pointee must keep all but ONE star.

use xiom.mem;

fn main() -> Int {
  var s: Str = "a";
  let old = mem.replace[Str](&mut s, "b");
  if s != "b" { return 1; }
  if old != "a" { return 2; }

  // Int path keeps working (single-star pointee).
  var n: Int = 7;
  let old_n = mem.replace[Int](&mut n, 9);
  if n != 9 { return 3; }
  if old_n != 7 { return 4; }
  return 0;
}
