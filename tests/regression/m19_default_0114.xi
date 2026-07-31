module regression.m19_default_0114

interface Incrementer {
  fn next_id(&self) -> Int { return id() + 1; }
  fn id(&self) -> Int;
}

type Entity = { uid: Int; }

fn Entity.next_id(self) -> Int { return self.id() + 1; }


fn Entity.id(&self) -> Int { return uid; }

fn main() -> Int {
  var e: Entity = Entity{ uid: 500 };
  if e.id() != 500 { return 1; }
  if e.next_id() != 501 { return 2; }
  return 0;
}
