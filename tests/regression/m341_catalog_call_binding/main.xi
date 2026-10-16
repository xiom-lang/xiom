// m341 fixture (REL-20261015-1358-engine W1.1): same-leaf catalog call
// binding from a SECONDARY module. jm.mod_json does `use
// xiom.serialize.json;` and calls bare json_string/json_number --
// pre-fix the emitted call bound the PARENT module's same-leaf fn
// xiom.serialize.json_string (Str) instead of the imported
// serialize.json.json_string (JsonValue), so json_stringify read a Str
// buffer as a JsonValue struct and the program crashed (0xC0000409 /
// 0xC0000005). Distinct exit codes per shape; the entry-module control
// (`json_stringify(json_string("x"))` in THIS file) was green pre-fix.
module main

use jm.mod_json;
use xiom.serialize.json;
use xiom.io;

fn main() -> Int {
    // (control) entry-module json path -- green pre-fix.
    if json_stringify(json_string("x")) != "\"x\"" { return 1; }
    // Secondary-module calls -- pre-fix crash / garbage.
    if s_str() != "\"x\"" { return 2; }
    if s_int() != "42" { return 3; }
    if s_bound() != "\"x\"" { return 4; }
    return 0;
}
