// C-PULSE request-cycle memory probe (REL-20261103-loop-drop).
//
// Reproduces the runtime-agent witness: a fresh 64 KiB Vec[UInt8] is built
// per request-loop iteration and passed by reference to server_parse_request.
// The stdlib function carries #[borrows_vec], so the Vec stays owned by the
// caller and the loop-body back-edge frees it (IR: the drop lives inside
// while_body). server_build_response carries #[owns_str], so the response
// buffer is freed at the same point.
//
// Residual (reported separately): `let r = server_parse_request(&buf)` is
// never consumed, and Option[ServerRequest] return-payload aggregates have no
// drop glue yet, so ~0.9 KiB/iter still grows. That is a distinct
// aggregate-return slice, not the 64 KiB loop-local Vec.
module c_pulse_request_cycle

use xiom.net.server;
use xiom.io;
use xiom.convert;

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
    let r = server_parse_request(&buf);
    let resp = server_build_response(200, "hello");
    total = total + resp.len();
    i = i + 1;
  }
  return total;
}

fn main() -> Int {
  io.println("c_pulse_request_cycle=" + convert.int_to_string(run(10000)));
  io.flush_stdout();
  return 0;
}
