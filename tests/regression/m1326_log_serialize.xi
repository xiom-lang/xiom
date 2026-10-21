// m1326 (REL-20261020-1326-stdlib): a struct field whose type is a plain
// enum must not be compared with `icmp eq %struct.Enum`.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `xiom.log.json` + `xiom.serialize` in one program monomorphises
// Map.insert_LogEntry_JsonValue; its key comparison walks LogEntry fields,
// and `level: LogLevel` emitted `icmp eq %struct.LogLevel` (invalid IR:
// "icmp requires integer operands"). The structural fallback now compares
// the enum discriminant when the enum has no derived `.eq`.

module m1326_log_serialize

use xiom.log.json;
use xiom.serialize;
use xiom.io;

fn main() -> Int {
  let line = json.log_json_entry(2, "hello");
  io.println(line);
  if line.len() < 40 { return 1; }

  let parsed = json.log_json_parse("{\"timestamp\":\"t\",\"level\":\"INFO\",\"message\":\"m\",\"thread_id\":1}");
  if !parsed.is_ok { return 2; }
  if parsed.value.level != 2 { return 3; }

  if !serialize.is_valid_json("{\"a\":1}") { return 4; }
  io.println("m1326 ok");
  return 0;
}
