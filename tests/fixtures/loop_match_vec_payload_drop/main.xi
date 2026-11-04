// Loop / match-bound Vec payload drop probe (REL-20261103-loop-drop).
//
// `outer.pop()` on an owned Vec[Vec[Int]] boxes the removed element header
// (malloc + memcpy of the 32-byte %struct.Vec). The match arm binds that box;
// the arm-scope drop must free the element's data buffer AND the header box.
// The container's own exit drop walks only the *current* len, so the removed
// slot is never freed twice. Pre-fix the box and its 8 KiB buffer leaked every
// iteration (ledger 19643).
module loop_match_vec_payload_drop

use xiom.io;
use xiom.convert;

fn run(n: Int) -> Int {
  var total = 0;
  var i = 0;
  while i < n {
    var outer = Vec[Vec[Int]].new();
    var inner = Vec[Int].with_capacity(1024);
    var j = 0;
    while j < 1024 {
      inner.push(j);
      j = j + 1;
    }
    outer.push(inner);
    match outer.pop() {
      Some(v) => {
        total = total + v.len();
      }
      None => {
        total = total + 1;
      }
    }
    i = i + 1;
  }
  return total;
}

fn main() -> Int {
  io.println("loop_match_vec_payload_drop=" + convert.int_to_string(run(100000)));
  io.flush_stdout();
  return 0;
}
