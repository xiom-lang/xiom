module regression.m19_default_0037

interface Priced {
  fn total(&self) -> Int { return price() + tax(); }
  fn price(&self) -> Int;
  fn tax(&self) -> Int;
}

type Item = { price_val: Int; tax_val: Int; }

fn Item.price(&self) -> Int { return price_val; }

fn Item.tax(&self) -> Int { return tax_val; }

fn main() -> Int {
  var i: Item = Item{ price_val: 100, tax_val: 20 };
  if i.price() == 100 && i.tax() == 20 && i.total() == 120 { return 0; }
  return 1;
}
