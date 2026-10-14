// m317 fixture (REL-20261013-2256-engine + -2324-engine-2 + -2324-engine-3):
// the constant-surface family module surface. The entry covers every shape
// with a distinct exit code so a regression pinpoints which one broke.
//
// Shape (a): a file-scope const SHADOWING an imported constant name. The
// initializer reads the QUALIFIED imported value (`xiom.m317x.xlib.VALUE
// + 1`), which pre-fix recursed through the codegen's bare-leaf const
// substitution until the compiler stack-overflowed (the engine's "hangs
// with no diagnostic"); the bare `VALUE` must stay the LOCAL 12 while
// `xlib.VALUE` stays the module's 11.
use xiom.m317x.xlib;
use xiom.m317x.xfacade;
use xiom.m317x.xbare;
use xiom.m317x.xglobber;
use xiom.m317x.xdleft;
use xiom.m317x.xdright;

// Shape (a): file-scope const shadowing the imported constant name VALUE.
const VALUE: Int = xiom.m317x.xlib.VALUE + 1;

fn main() -> Int {
  // (a) local shadow keeps its own value; the qualified read keeps the
  // module's value (pre-fix the qualified read returned the local 12).
  if VALUE != 12 { return 1; }
  if xlib.VALUE != 11 { return 2; }
  if xlib.xlib_get() != 11 { return 3; }
  // (c) `pub use` re-export carries CONSTANTS: qualified through the facade,
  // bare through the facade's injected surface, and a fn of the
  // re-exported module.
  if xfacade.FVALUE != 33 { return 4; }
  if FVALUE != 33 { return 5; }
  if xfacade.fmod_get() != 33 { return 6; }
  // (e) diamond: two facets re-export the same common module.
  if DCOMMON != 22 { return 7; }
  if xdleft.DCOMMON != 22 { return 8; }
  // (b) bare imported constants INSIDE module files (multi-use + glob +
  // aliased imports; pre-fix the catalog-body use-skip dropped the const
  // import for these shapes).
  if xbare.bare_sum() != 63 { return 9; }
  if xglobber.glob_get() != 33 { return 10; }
  if xbare.bare_alias_get() != 33 { return 11; }
  // (d) `pub var`: a module-level var KEEPS its `pub` export (pre-fix the
  // parser dropped it silently) -- read and fn-value call through the
  // module path.
  if xlib.counter != 7 { return 12; }
  if xlib.call_handler(3) != 6 { return 13; }
  if xlib.handler(3) != 6 { return 14; }
  return 0;
}
