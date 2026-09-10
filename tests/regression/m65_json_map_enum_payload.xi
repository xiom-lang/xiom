// M65 (json heap layer): Map whose VALUE type is a big ENUM payload struct
// (JsonValue = 112 bytes). PART 1 (landed): Vec.new() inside a mono'd
// generic Map ctor kept the RAW generic param ("V") as the expression-level
// type arg, so the values buffer was sized at 8 bytes per slot (truncation);
// the ctor now substitutes through mono.current_type_map (stride 112 --
// pinned by e2e_m65a_json_values_stride's IR check).
// PART 2 (Stage 2c, pending): the map VALUE READ still scalar-loads 8 bytes
// and inttoptrs them as a %struct.JsonValue*; json_get/json_type chains on
// non-string values AV until payload-type propagation lands. This file is
// the run-time regression for Part 2 -- register e2e_m65_json_map_enum_
// payload (compile_and_run == Some(0)) when it lands.
module m65_json_map_enum_payload

use xiom.serialize.json;
use xiom.io;

fn main() -> Int {
  var doc = json.json_parse("{\"name\":\"xiom\",\"n\":42,\"flags\":[true,false,null],\"nested\":{\"deep\":\"v\"}}");
  match doc {
    Ok(v) => {
      match json.json_get(v, "name") {
        Some(_s) => { io.println("m65 OK"); return 0; },
        None => { io.println("m65 missing"); return 1; },
      }
    },
    Err(e) => { io.println("m65 err: " + e); return 2; },
  }
}
