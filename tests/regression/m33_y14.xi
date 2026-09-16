// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Y14: triple impl interface + generic enum + struct field access + match + contract + module + diff
type Product = { name: Int; price: Int; qty: Int; }
enum DiscountKind { Percent(pct: Int), Fixed(amt: Int), None }
fn price[T](p: Product, d: DiscountKind) -> Int
  requires: p.price >= 0
  requires: p.qty >= 0
  ensures: result >= 0
{
  match d {
    Percent(pct) => p.price - (p.price * pct / 100),
    Fixed(amt) => if p.price > amt { p.price - amt } else { 0 },
    None => p.price,
  }
}
fn total_raw(p: Product) -> Int { return p.price * p.qty; }
interface Priceable { fn cost(self) -> Int; }
interface Stockable { fn stock(self) -> Int; }
impl Priceable for Product {
  fn cost(self) -> Int { return self.price; }
}
impl Stockable for Product {
  fn stock(self) -> Int { return self.qty; }
}
module store {
  pub fn do_price(p: Product, d: DiscountKind) -> Int { return price(p, d); }
  pub fn do_total(p: Product) -> Int { return total_raw(p); }
  pub fn via_cost(p: Product) -> Int { return p.cost(); }
}
use store.do_price;
use store.do_total;
use store.via_cost;
enum Path { Price, Raw, Cost }
fn resolve(p: Path, prod: Product, disc: DiscountKind) -> Int {
  match p { Price => do_price(prod, disc), Raw => do_total(prod), Cost => via_cost(prod), }
}
fn main() -> Int {
  var prod = Product{ name: 1; price: 100; qty: 2; };
  var r1 = resolve(Path.Price, prod, DiscountKind.Percent(10));
  var r2 = resolve(Path.Cost, prod, DiscountKind.None);
  var r3 = resolve(Path.Raw, prod, DiscountKind.None);
  if r1 == 90 && r2 == 100 && r3 == 200 { return 0; }
  return 1;
}
