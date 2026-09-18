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

pub fn Box.is_sealed[T]() -> Bool {
  return sealed;
}
