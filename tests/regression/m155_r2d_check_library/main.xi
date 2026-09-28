// m155 (benchmark relay R-2d): a LIBRARY-style file (top-level declarations,
// no `fn main`) must type-check via `--check`. The check path used to apply
// implicit-main wrapping to any file without `fn main`, putting every
// declaration inside a fn body and failing with a bogus P001 at the first
// `requires:` clause -- while a full compile of the same file succeeded.
module m155_r2d_check_library;

use xiom.io;

fn nonneg(n: Int) -> Int
    requires: n >= 0
{
  return n;
}

fn label(v: Int) -> Str {
  return "v=" + nonneg(v).to_str();
}
