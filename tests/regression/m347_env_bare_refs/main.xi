// m347 (REL-20261016-2000-stdlib): a BARE in-module reference to the
// declaring module's own platform constant must carry the TARGET value, not
// the stdlib's literal initializer. Pre-fix the injected consts registered
// the literal under the module-qualified key (`env.FAMILY`) that
// `lookup_const_keyed` consults FIRST for an in-module bare ref, while
// qualified refs rewrote to the target: on Linux `env.FAMILY` read "unix"
// from outside while a bare FAMILY check inside xiom.env still matched
// "windows" (path_separator "\\", AppData/* dirs -- the stdlib wave-164
// relay). Qualified-vs-bare agreement is host-agnostic and red pre-fix on
// non-Windows hosts (exit 22; the Windows-observable lock is the sibling
// `--target arm` IR leg, which sees the literal "windows"/"x86_64" in the
// bare fn bodies pre-fix).
module m347_env_bare_refs

use xiom.os.env;

fn main() -> Int {
  let q = env.FAMILY;                      // cross-module qualified (site rewrite)
  let s = env.probe_qualified_family();    // self-qualified in-module (site rewrite)
  let b = env.probe_bare_family();         // bare in-module (registration value)
  let bo = env.probe_bare_os();            // bare in-module (registration value)
  if b != q { return 22; }                 // bare must agree with qualified
  if s != q { return 23; }
  if q == "windows" {
    if b != "windows" { return 24; }
    if bo != "windows" { return 25; }
    if s != "windows" { return 26; }
  } else {
    if q != "unix" { return 27; }
    if b != "unix" { return 28; }
    if bo != "linux" { return 29; }
    if s != "unix" { return 30; }
  };
  return 0;
}
