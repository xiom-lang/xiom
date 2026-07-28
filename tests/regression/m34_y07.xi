// M34-Y07: Result inside match + generic dispatch + derive + compound assign + module
type Item = { id: Int; price: Int; }
enum Action { Buy, Sell, Hold }
fn dispatch[T](it: Item, act: Action) -> Result[Int, Str]
  requires: it.id >= 0
  requires: it.price >= 0
{
  match act {
    Buy => Ok(it.price + 5),
    Sell => {
      var v = it.price;
      v = v - 3;
      if v > 0 { return Ok(v); }
      else { return Err("bad"); }
    }
    Hold => Ok(it.price),
  }
}
module shop {
  pub fn do_dispatch(it: Item, act: Action) -> Result[Int, Str] { return dispatch(it, act); }
  pub fn direct_price(it: Item) -> Int { return it.price; }
}
use shop.do_dispatch;
use shop.direct_price;
fn main() -> Int {
  var it = Item{ id: 1; price: 10; };
  match do_dispatch(it, Action.Buy) {
    Ok(v) => { if v != 15 { return 1; } }
    Err(_) => { return 2; }
  }
  match do_dispatch(it, Action.Sell) {
    Ok(v) => { if v != 7 { return 3; } }
    Err(_) => { return 4; }
  }
  match do_dispatch(it, Action.Hold) {
    Ok(v) => { if v != 10 { return 5; } }
    Err(_) => { return 6; }
  }
  return 0;
}
