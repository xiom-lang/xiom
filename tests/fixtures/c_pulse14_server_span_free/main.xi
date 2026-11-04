// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// C-PULSE-14 stdlib half: server_parse_request copies every request-line
// field and header name/value with `Str.from_cstring`, which allocates an
// owned copy (m211), so the temporary `malloc` buffer must be freed. Pre-fix
// both `_str_from_byte_span` and `_str_lower_from_byte_span` leaked the
// source buffer (~145 B per 6-header parse). This fixture locks the parse
// semantics over a long loop; run it under `--sanitize=address` (or with a
// leak checker) to catch an invalid/double free introduced by the fix.
module c_pulse14_server_span_free;

use xiom.net.server;
use xiom.string.slice;

const ITERS: Int = 50000;

fn main() -> Int {
  let head = "GET /health?x=1 HTTP/1.1\r\nHost: example.com\r\nAccept: */*\r\nUser-Agent: pulse\r\nContent-Length: 0\r\nX-Trace: abcdef\r\n\r\n";
  var req = slice.str_bytes(head);
  var acc = 0;
  var i = 0;
  while i < ITERS {
    match server.server_parse_request(&req) {
      Some(r) => {
        if r.method != "GET" { return 1; }
        if r.target != "/health?x=1" { return 2; }
        if r.version != "HTTP/1.1" { return 3; }
        if r.headers.len() != 5 { return 4; }
        if r.headers[0].0 != "host" { return 5; }
        if r.headers[4].0 != "x-trace" { return 6; }
        if r.headers[4].1 != "abcdef" { return 7; }
        acc = acc + r.method.len() + r.target.len() + r.headers.len();
      },
      None => { return 8; },
    }
    i = i + 1;
  }
  if acc < ITERS { return 9; }
  return 0;
}
