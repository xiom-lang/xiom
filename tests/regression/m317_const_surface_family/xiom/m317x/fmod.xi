// m317 fixture module: re-export target of the facade (shape c). NOT
// imported directly by main -- `FVALUE` is only reachable through the
// facade's `pub use` re-export.
module xiom.m317x.fmod;

pub const FVALUE: Int = 33;

pub fn fmod_get() -> Int {
  return FVALUE;
}
