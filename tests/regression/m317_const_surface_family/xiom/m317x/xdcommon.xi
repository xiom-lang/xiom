// m317 fixture module (shape e): the diamond's common module, reached by
// BOTH facets through `pub use` re-exports.
module xiom.m317x.xdcommon;

pub const DCOMMON: Int = 22;

pub fn dcommon_get() -> Int {
  return DCOMMON;
}
