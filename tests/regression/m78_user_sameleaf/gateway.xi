// m78 root: declares the delegating same-leaf module (beta.base32) and the
// gateway. All three files are passed on the COMMAND LINE (user program, no
// catalog): pre-fix the shim's `canon.encode` resolved to the caller's own
// qualified symbol (`call @beta.base32.encode`, self-recursion ->
// 0xC000001D) because alpha kept the bare symbol and the leaf/alias maps
// collapsed the two `base32` modules.
module gateway
use beta.base32 as b32;
use alpha.base32 as canon_direct;

fn main() -> Int {
  if b32.encode(10) != 20 { return 1; }
  if b32.name() != "alpha" { return 2; }
  if canon_direct.encode(11) != 22 { return 3; }
  if canon_direct.name() != "alpha" { return 4; }
  return 0;
}
