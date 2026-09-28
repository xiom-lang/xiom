// m150 (R-5 benchmark relay): `use xiom.num;` together with `use xiom.ffi.dl;`
// used to fail the compile with two bogus T002 findings against the
// xiom.math.primitives catalog body. xiom.ffi.c (loaded through the xiom.ffi
// parent module) declares a PRIVATE libc `extern "C" fn abs`; the
// extern-name set is global, so the name-based confinement gate fired on
// primitives' internal `abs(x)` calls even though they resolve to its own
// `pub fn abs`. The finding's span (191/223) also printed as a user-file
// line. This fixture keeps the import-closure shape that triggered the T001
// and exercises the dl module's public API end to end.
//
// The relay's exact program (with `num.parse_int(...)`) is locked at CHECK
// level by tests/regression/m150_dl_num_parse + the checker_locks entry
// m150_dl_num_parse_checks_clean. Its RUN lock lands with the separate
// io.parse_int `is_empty` C001 batch (codegen, pre-existing): until that
// lands, compiling that shape fails at codegen, so it cannot be an e2e lock.
module m150_dl_num_catalog_abs;

use xiom.num;
use xiom.ffi.dl;
use xiom.io;

fn main() -> Int {
  io.println("r5");
  match dl_open("xiom_missing_module_xyz") {
    Ok(h) => { let _ = h; return 3; },
    Err(e) => { let _ = e; },
  }
  return 0;
}
