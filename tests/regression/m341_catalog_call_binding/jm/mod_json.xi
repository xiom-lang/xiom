// m341 fixture module (W1.1): calls the imported json module's own
// same-leaf fns BARE. The m162 resolved-target record only fired for
// explicit ITEM imports; a SUBMODULE surface import (`use
// xiom.serialize.json;`) records the module alias instead, so no record
// was emitted and codegen's keep-first bare alias bound the parent
// module's Str-returning json_string. Red-pre: rc 0xC0000409 / 0xC0000005
// before any output.
module jm.mod_json

use xiom.serialize.json;

pub fn s_str() -> Str {
    return json_stringify(json_string("x"));
}

pub fn s_int() -> Str {
    return json_stringify(json_number(42.0));
}

pub fn s_bound() -> Str {
    var v = json_string("x");
    return json_stringify(v);
}
