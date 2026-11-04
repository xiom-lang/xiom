// m375 / R3: a direct `let p = json_parse(s)` binding owns the parsed
// JsonValue tree and reclaims it at scope exit. This fixture nests
// object -> array -> object -> string (plus an empty string, which is a
// literal and must NOT be freed) and runs the parse repeatedly.
//
// Green = rc 0 and `json_tree_reclaim_ok`. The reclamation itself is proven
// by peak RSS (stdlib/tools/json_peak.ps1) and by an ASan run of the same
// fixture (no heap-use-after-free / invalid-free).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m375_json_parse_tree_reclaim

use xiom.io;
use xiom.serialize.json;

fn main() -> Int {
  let s = "{\"outer\":[{\"inner\":\"wave-1704\"},{\"n\":7}],\"empty\":\"\"}";
  var iter = 0;
  var acc = 0;
  while iter < 1000 {
    let p = json.json_parse(s);
    if p.is_err { return 1; }
    acc = acc + s.len();
    iter = iter + 1;
  }
  if acc < 1000 { return 2; }
  io.println("json_tree_reclaim_ok");
  return 0;
}
