// R39 lock gateway. Compiled through the package graph (the file is the only
// command-line source; alpha/beta arrive as catalog modules), which is the
// path that used to lose the module qualifier on injected type decls.
module m84.main

use m84.alpha as am;
use m84.beta as bm;

fn main() -> Int {
  var a = am.make();
  var b = bm.make();
  if am.total(&a) != 7 { return 1; }
  if bm.total(&b) != 10 { return 2; }
  return 0;
}
