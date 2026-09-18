// m88 (R46): Box[T] with TWO fields (conflicting shape). Pre-fix both
// generic Box declarations injected as one bare `%struct.Box` (the 1-field
// definition won) and `pack` GEP'd field 1 of it -> clang reject.
module m88.hard

pub type Box[T] = {
  item: T;
  sealed: Bool;
}

pub fn Box.pack[T](v: T) -> Box[T] {
  return Box[T]{ item: v, sealed: true };
}

pub fn Box.item_of[T]() -> T {
  return item;
}

// Non-generic wrappers keep the instantiation module-local (the lock targets
// the same-leaf layout collision, not cross-module generic method mono).
pub fn make_packed() -> Box[Int] {
  return Box.pack[Int](42);
}

pub fn packed_item(b: &Box[Int]) -> Int {
  return b.item_of[Int]();
}
