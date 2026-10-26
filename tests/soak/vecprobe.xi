// Engine vecprobe (REL: m375 function/loop-scope reclaim).
// PROBE_VEC_MODE=0 loop-body Vec; PROBE_VEC_MODE=1 Vec inside a called fn.
// Element writes/reads (v[0] = i) must not count as a Vec escape.
module vecprobe

use xiom.io;
use xiom.convert;
use xiom.os.env;
use xiom.convert.parse;

fn env_int(name: Str, def: Int) -> Int {
  match get_var(name) {
    Ok(v) => {
      match parse_int_opt(v) {
        Some(n) => { return n; }
        None => { return def; }
      };
    }
    Err(e) => { return def; }
  };
}

fn make_one(i: Int) -> Int {
  var v = Vec[Int].with_len(64);
  v[0] = i;
  return v[0];
}

fn main() -> Int {
  let n = env_int("PROBE_VEC_N", 100000);
  let mode = env_int("PROBE_VEC_MODE", 0);
  var i = 0;
  var sink = 0;
  if mode == 1 {
    while i < n {
      sink = sink + make_one(i);
      i = i + 1;
    }
  } else {
    while i < n {
      var v = Vec[Int].with_len(64);
      v[0] = i;
      sink = sink + v[0];
      i = i + 1;
    }
  }
  println("vec probe done mode=" + int_to_string(mode) + " n=" + int_to_string(n) + " sink=" + int_to_string(sink));
  return 0;
}
