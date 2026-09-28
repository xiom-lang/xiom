// m150 companion (R-5): the benchmark relay's exact program. Its CHECK must
// stay clean (checker_locks m150_dl_num_parse_checks_clean) -- the R-5 bug
// was a check-phase false positive, so this is the direct lock on it.
//
// It is NOT a run-based lock yet: codegen of the num + ffi closure pulls
// io.parse_int, whose `trimmed.is_empty()` emits an undefined bare
// `@is_empty` (C001, pre-existing, filed for its own batch). When that
// batch lands, promote this fixture to e2e (and/or fold it back into
// m150_dl_num_catalog_abs).
module m150_dl_num_parse;

use xiom.num;
use xiom.ffi.dl;
use xiom.io;

fn main() {
  let n = num.parse_int("5").unwrap_or(0);
  io.println(n.to_str());
  match dl_open("xiom_missing_module_xyz") {
    Ok(h) => { let _ = h; },
    Err(e) => { let _ = e; },
  }
}
