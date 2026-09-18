module m88.generics

pub type Box[T] = {
  value: T;
}

pub fn Box.new[T](v: T) -> Box[T] {
  return Box[T]{ value: v };
}

pub fn Box.value_of[T]() -> T {
  return value;
}
