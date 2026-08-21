// M32-X12: Combinatorial + Differential -- struct+enum combination with dual equivalence
type Rect = { x: Int; y: Int; w: Int; h: Int; }
enum AreaMethod { Full, Decomposed }
fn area_full(r: Rect) -> Int {
  return r.w * r.h;
}
fn area_decomposed(r: Rect) -> Int {
  var corner = Rect{ x: r.x; y: r.y; w: 1; h: 1; };
  var body_w = r.w * r.h;
  return body_w;
}
fn area(m: AreaMethod, r: Rect) -> Int {
  match m { Full => area_full(r), Decomposed => area_decomposed(r), }
}
fn main() -> Int {
  var r1 = Rect{ x: 0; y: 0; w: 5; h: 7; };
  var r2 = Rect{ x: 2; y: 3; w: 10; h: 4; };
  var a1 = area(AreaMethod.Full, r1);
  var b1 = area(AreaMethod.Decomposed, r1);
  var a2 = area(AreaMethod.Full, r2);
  var b2 = area(AreaMethod.Decomposed, r2);
  if a1 == b1 && a2 == b2 && a1 == 35 && a2 == 40 { return 0; }
  return 1;
}
