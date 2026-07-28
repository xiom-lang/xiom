// M32-G07: Method on type via interface and generic constraint
interface Reader { fn read(self) -> Int; }
type Book = { page: Int; }
impl Reader for Book {
  fn read(self) -> Int { return self.page; }
}
fn use_reader(x: Book) -> Int { return x.read(); }
fn main() -> Int {
  var b = Book{ page: 77 };
  if use_reader(b) != 77 { return 1; }
  return 0;
}
