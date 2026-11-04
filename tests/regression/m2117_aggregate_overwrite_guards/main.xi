// REL-2117 negative guards. Both shapes MUST stay leak-only:
//   * trace ring: `var out = r; out.events = fresh` shallow-copies the
//     aggregate, so out.events aliases r.events.
//   * field alias: `let x = d.events; d.events = fresh` reads the Vec header
//     into a binding before the overwrite.
// A blind field free in either shape is a use-after-free / invalid free.
module m2117_aggregate_overwrite_guards

use xiom.io;
use xiom.convert;

type Ring = {
  events: Vec[Int];
  n: Int;
}

#[owns_return]
fn mk(n: Int) -> Vec[Int] {
  var b = Vec[Int].new();
  var i = 0;
  while i < n {
    b.push(i);
    i = i + 1;
  }
  return b;
}

fn trace_ring(iters: Int) -> Int {
  var r = Ring{ events: mk(16); n: 0 };
  var out = r;
  var i = 0;
  while i < iters {
    out.events = mk(16);
    i = i + 1;
  }
  return out.events.len();
}

fn field_alias(iters: Int) -> Int {
  var d = Ring{ events: mk(16); n: 0 };
  var x = d.events;
  var i = 0;
  while i < iters {
    d.events = mk(16);
    i = i + 1;
  }
  return x.len();
}

fn main() -> Int {
  var a = trace_ring(1000);
  var b = field_alias(1000);
  io.println("guards=" + convert.int_to_string(a + b));
  return 0;
}
