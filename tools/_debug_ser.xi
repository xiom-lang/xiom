module smoke_serialize
use xiom.serialize;
use xiom.collections;
use xiom.convert;
fn main() -> Int {
  let b = xiom.serialize.json_bool(true);
  io.println("json_bool: " + b);
  let n = xiom.serialize.json_null();
  io.println("json_null: " + n);
  let r = xiom.serialize.parse_json("[1, 2, 3]");
  io.println("parse_json is_ok: " + convert.int_to_string(r.is_ok as Int));
  if r.is_ok { return 0; }
  return 1;
}
