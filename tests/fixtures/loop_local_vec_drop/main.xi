// Loop-body drop-glue probe (REL-20261103-loop-drop).
//
// Every iteration allocates a 64 KiB Vec[UInt8] (request-cycle shape), fills
// it in an inner loop, borrows it through a #[borrows_vec] consumer, allocates
// a fresh Str, then rebinds the Vec to a second fresh buffer. All five
// allocations must be reclaimed at the loop-body back-edge: the Vec data
// buffer (twice: rebind + body exit), the Str buffer, and no double free at
// function exit. Pre-fix the per-iteration buffer leaked (~65 KiB/iter).
module loop_local_vec_drop

use xiom.io;
use xiom.convert;

// Reads the request bytes without retaining the buffer. The contract keeps
// `buf` owned by the caller so the per-iteration drop stays armed.
#[borrows_vec]
fn scan_request(bytes: &Vec[UInt8]) -> Int {
  var i = 0;
  var sum = 0;
  while i < bytes.len() {
    sum = sum + (bytes[i] as Int);
    i = i + 1;
  }
  return sum;
}

fn run(n: Int) -> Int {
  let req = "POST /insert HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 163\r\n\r\n";
  var total = 0;
  var i = 0;
  while i < n {
    var buf = Vec[UInt8].with_capacity(65536);
    var j = 0;
    while j < req.len() {
      buf.push(req.char_at(j) as UInt8);
      j = j + 1;
    }
    // Cross the with_capacity boundary so a realloc replaces the buffer.
    while j < 4096 {
      buf.push(7 as UInt8);
      j = j + 1;
    }
    let tag = "req-" + "ok";
    total = total + scan_request(&buf) + tag.len();
    // Rebind inside the loop: the replaced buffer must be freed here.
    buf = Vec[UInt8].with_capacity(65536);
    buf.push(1 as UInt8);
    total = total + buf.len();
    i = i + 1;
  }
  return total;
}

fn main() -> Int {
  io.println("loop_local_vec_drop=" + convert.int_to_string(run(100000)));
  io.flush_stdout();
  return 0;
}
