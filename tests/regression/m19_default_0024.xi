module regression.m19_default_0024

interface Introducible {
  fn intro(&self) -> Str { return "I am " + name() + " from " + place(); }
  fn name(&self) -> Str;
  fn place(&self) -> Str;
}

type Resident = { who: Str; where: Str; }

fn Resident.intro(self) -> Str { return "I am " + self.name() + " from " + self.place(); }


fn Resident.name(&self) -> Str { return who; }

fn Resident.place(&self) -> Str { return where; }

fn main() -> Int {
  var r: Resident = Resident{ who: "Zoe", where: "Paris" };
  if r.name() == "Zoe" && r.place() == "Paris" && r.intro() == "I am Zoe from Paris" { return 0; }
  return 1;
}
