use xiom.io;

type Stack = {
  items: Vec[Int];
}

fn Stack.pop(&mut self) -> Option[Int]
  requires: self.items.len() > 0;
  ensures: self.items.len() == self@pre.items.len() - 1;
{
  let item = self.items[self.items.len() - 1];
  self.items.remove(self.items.len() - 1);
  return Some(item);
}

fn Stack.peek(self) -> Option[Int]
  requires: self.items.len() > 0;
{
  return Some(self.items[self.items.len() - 1]);
}

fn main() -> Int {
  let stack = Stack{ items: Vec::new() };
  stack.items.push(10);
  stack.items.push(20);
  stack.items.push(30);
  stack.pop();
  let top = stack.peek();
  match top {
    Some(val) => { return val; };
    None => { return -1; };
  }
}