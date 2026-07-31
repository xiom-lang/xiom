module regression.m19_default_0036

interface Reporter {
  fn report(&self) -> Int { return size() * 2; }
  fn size(&self) -> Int;
}

type Container = { len: Int; }

fn Container.report(self) -> Int { return self.size() * 2; }


fn Container.size(&self) -> Int { return len; }

fn main() -> Int {
  var c: Container = Container{ len: 3 };
  if c.size() == 3 && c.report() == 6 { return 0; }
  return 1;
}
