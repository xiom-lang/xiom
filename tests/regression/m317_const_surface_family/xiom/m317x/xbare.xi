// m317 fixture module (shape b): a module file that imports TWO
// constant-declaring modules and reads their constants BARE. Pre-fix the
// catalog-body use-skip heuristic dropped both imports (the bound module
// names never appear in the body) and the bare reads were T001.
module xiom.m317x.xbare;

use xiom.m317x.xlib;
use xiom.m317x.xdcommon;

pub fn bare_sum() -> Int {
  return AVALUE + DCOMMON;
}

// Aliased import of a const module: pre-fix the bound alias name was never
// referenced and the skip dropped the use too.
use xiom.m317x.fmod as fm;

pub fn bare_alias_get() -> Int {
  return FVALUE;
}
