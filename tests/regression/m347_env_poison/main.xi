// m347 poison lock (REL-20261016-2000 second defect): the R65 qualified-site
// rewrite used to SEED the bare leaf (`FAMILY` -> platform value) on the
// first qualified env.FAMILY read, clobbering a user program-level
// `const FAMILY` for every later bare read -- a silent miscompile (pre-fix
// this fixture exits 3 on Windows AND WSL; `b` read the platform value).
// The const and main stay TOP-LEVEL (no `module` header): a module wrapper
// registers a qualified `{module}.FAMILY` key that wins the lookup, which
// masks the bare-slot clobber this lock exists to catch.
// The seed is gone; the user const must stay untouched (exit 0).

use xiom.os.env;

const FAMILY: Str = "mine";

fn main() -> Int {
  let a = FAMILY;        // before any qualified read
  let q = env.FAMILY;    // qualified read (site rewrite)
  let b = FAMILY;        // after the qualified read -- must still be "mine"
  if a != "mine" { return 1; }
  if b != "mine" { return 3; }
  if q.len() == 0 { return 4; }
  return 0;
}
