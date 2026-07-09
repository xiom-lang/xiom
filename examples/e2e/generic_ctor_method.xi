// E2E: generic method dispatch on a user-defined type (in-module, no stdlib)
module e2e_generic_ctor_method

type Box[T] = { item: T; }

fn Box.make[T](item: T) -> Box[T] { return Box[T]{ item: item; }; }
fn Box.get[T](self) -> T { return item; }
fn Box.replace[T](self, new_item: T) -> Box[T] { return Box[T]{ item: new_item; }; }

fn main() -> Int {
  var b = Box.make[Int](42);
  if b.get() != 42 { return 1; }
  b = b.replace(99);
  if b.get() != 99 { return 2; }
  return 0;
}
