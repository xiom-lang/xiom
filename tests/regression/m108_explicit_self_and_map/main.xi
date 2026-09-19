// R52 lock (playground L5-09/L5-43): Map[Str,Str] payload binds as i8*; a
// redundant explicit `&receiver` argument is ignored instead of shifting the
// real argument.
use xiom.io;

type Stack[T] = {
  items: Vec[T];
}

fn Stack.new[T]() -> Stack[T] {
  return Stack[T]{ items: Vec[T].new() };
}

fn Stack.push[T](&mut self, item: T) {
  self.items.push(item);
}

fn Stack.pop[T](&mut self) -> Option[T] {
  return self.items.pop();
}

fn main() -> Int {
  let m: Map[Str, Str] = Map[Str, Str].new();
  m.insert("Mom", "555-0100");
  let num = match m.get("Mom") { Some(v) => v, None => "?" };
  if num != "555-0100" { return 1; }
  let missing = match m.get("Zed") { Some(v) => v, None => "NONE" };
  if missing != "NONE" { return 2; }

  var s: Stack[Str] = Stack[Str].new();
  s.push(&mut s, "Alice");
  s.push(&mut s, "Bob");
  let top = match s.pop(&mut s) { Some(v) => v, None => "?" };
  if top != "Bob" { return 3; }
  let next = match s.pop(&mut s) { Some(v) => v, None => "?" };
  if next != "Alice" { return 4; }

  io.println("ok");
  return 0;
}
