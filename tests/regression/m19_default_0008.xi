module regression.m19_default_0008

interface Named {
  fn full(&self) -> Str { return first() + " " + last(); }
  fn first(&self) -> Str;
  fn last(&self) -> Str;
}

type Person = { fname: Str; lname: Str; }

fn Person.first(&self) -> Str { return fname; }

fn Person.last(&self) -> Str { return lname; }

fn main() -> Int {
  var p: Person = Person{ fname: "John", lname: "Doe" };
  if p.first() == "John" && p.last() == "Doe" && p.full() == "John Doe" { return 0; }
  return 1;
}
