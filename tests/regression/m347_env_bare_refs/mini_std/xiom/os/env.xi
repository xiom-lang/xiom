// m347 hermetic mini XIOM_STDLIB: the module under test is the stdlib env
// shape -- `xiom.env` with its own literal platform constants (the stdlib
// declares "windows"/"x86_64" as initializers) and fns that reference the
// constants INSIDE the declaring module:
//   - bare in-module refs (probe_bare_family/probe_bare_os) -- pre-fix
//     these kept the literals while qualified refs rewrote (the REL-20261016
//     -2000 defect),
//   - a self-qualified in-module ref (probe_qualified_family) -- the R65
//     site rewrite path that always worked.
// The tests point XIOM_STDLIB at this directory, so the compile is
// independent of the ambient checkout's env.xi.
module xiom.env

pub const OS: Str = "windows";
pub const ARCH: Str = "x86_64";
pub const FAMILY: Str = "windows";

pub fn probe_bare_family() -> Str {
  return FAMILY;
}

pub fn probe_bare_os() -> Str {
  return OS;
}

pub fn probe_qualified_family() -> Str {
  return xiom.env.FAMILY;
}
