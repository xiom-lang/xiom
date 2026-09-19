use xiom.io;
// R49 lock: L6-28 -- annotated generic-struct local keeps its type args
// (PriorityQueue[Task]) so pop() infers T=Task; Vec.get/unwrap must unbox
// the struct payload (was C001 / AV).
interface Priority {
  fn is_higher_than(&self, other: &Self) -> Bool;
}

type Task = {
  name: Str;
  urgency: Int;
}

fn Task.is_higher_than(&self, other: &Task) -> Bool {
  self.urgency > other.urgency
}

type PriorityQueue[T: Priority] = {
  items: Vec[T];
}

fn PriorityQueue.insert[T: Priority](&mut self, item: T) {
  var pos = 0;
  while pos < self.items.len() {
    if item.is_higher_than(self.items.get(pos).unwrap()) {
      break;
    };
    pos += 1;
  };
  self.items.insert(pos, item);
}

fn PriorityQueue.pop[T: Priority](&mut self) -> Option[T] {
  if self.items.len() == 0 {
    return None;
  };
  let last = self.items.len() - 1;
  let result = self.items.get(last).unwrap().clone();
  self.items.remove(last);
  Some(result)
}

fn main() -> Int {
  var pq: PriorityQueue[Task] = PriorityQueue{ items: Vec[Task].new() };
  pq.insert(Task{ name: "Fix bug", urgency: 3 });
  pq.insert(Task{ name: "Write docs", urgency: 1 });
  pq.insert(Task{ name: "Deploy", urgency: 5 });
  while pq.items.len() > 0 {
    let t = pq.pop().unwrap();
    io.println(t.name.to_str());
  };
  return 0;
}
