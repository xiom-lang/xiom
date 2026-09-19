use xiom.io;

use xiom.string;
type Queue[T] = {
  items: Vec[T];
}

fn Queue.new[T]() -> Queue[T] {
  return Queue[T]{ items: Vec[T].new() };
}

fn Queue.enqueue[T](&mut self, item: T) {
  self.items.push(item);
}

fn Queue.dequeue[T](&mut self) -> Option[T] {
  if self.items.len() == 0 {
    return None;
  }
  let front = match self.items.get(0) { Some(v) => v, None => return None };
  let new_items: Vec[T] = Vec[T].new();
  let i = 1;
  while i < self.items.len() {
    let item = match self.items.get(i) { Some(v) => v, None => { i = i + 1; continue; } };
    new_items.push(item);
    i = i + 1;
  }
  while self.items.len() > 0 {
    self.items.pop();
  }
  i = 0;
  while i < new_items.len() {
    let item = match new_items.get(i) { Some(v) => v, None => { i = i + 1; continue; } };
    self.items.push(item);
    i = i + 1;
  }
  return Some(front);
}

fn main() -> Int {
  var q: Queue[Int] = Queue[Int].new();
  q.enqueue(10);
  q.enqueue(20);
  q.enqueue(30);

  let a = q.dequeue();
  let b = q.dequeue();
  let c = q.dequeue();
  io.println(match a { Some(v) => to_string(v), None => "?" });
  io.println(match b { Some(v) => to_string(v), None => "?" });
  io.println(match c { Some(v) => to_string(v), None => "?" });
  return 0;
}
