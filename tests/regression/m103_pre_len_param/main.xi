use xiom.io;

fn safe_pop(items: &mut Vec[Int]) -> Int
  requires: items.len() > 0;
  ensures: items.len() == items@pre.len() - 1;
{
  let last = items[items.len() - 1];
  items.pop();
  return last;
}

fn main() -> Int {
  var data = Vec[Int].new();
  data.push(10);
  data.push(20);
  data.push(30);
  let val = safe_pop(&mut data);
  return val;
}