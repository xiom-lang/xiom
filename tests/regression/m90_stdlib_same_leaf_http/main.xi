// R44 lock: same-leaf catalog TYPE collision across stdlib modules.
//
// On the pinned stdlib, `xiom.net.net.HttpResponse` (2 fields) and
// `xiom.net.http.HttpResponse` (3 fields) share the leaf `HttpResponse`.
// Catalog decls flatten into the program, so before R44 the bare
// `%struct.HttpResponse` (first module wins) had 2 fields while
// `http_parse_response` GEPed field 2 -- clang rejects the IR with
// "invalid getelementptr indices". With catalog modules participating in
// the R39 qualification (shape-conflict standard) both types emit
// module-qualified and the call compiles and runs.
//
// stdlib main renamed `net.net.HttpResponse` -> `NetHttpResponse`, so on
// newer checkouts this is a no-collision compile+run lock; the assertion
// (exit 0) holds either way.
module m90_stdlib_same_leaf_http

use xiom.net.http;

fn main() -> Int {
  let raw = "HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello";
  let r = http_parse_response(raw);
  match r {
    Ok(_) => { return 0; },
    Err(_) => { return 1; },
  }
}
