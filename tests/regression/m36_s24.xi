// M36-S24: Garbage collection simulation — mark/sweep phases
type GcObj = { id: Int; marked: Bool; reachable: Bool; size: Int; freed: Bool; }
fn make_obj(id: Int, sz: Int) -> GcObj {
  return GcObj{ id: id; marked: false; reachable: false; size: sz; freed: false; };
}
fn mark(obj: GcObj) -> GcObj {
  return GcObj{ id: obj.id; marked: true; reachable: true; size: obj.size; freed: obj.freed; };
}
fn sweep(obj: GcObj) -> GcObj {
  if !obj.marked && obj.reachable {
    return GcObj{ id: obj.id; marked: obj.marked; reachable: obj.reachable; size: obj.size; freed: true; };
  }
  return obj;
}
fn is_live(obj: GcObj) -> Bool {
  return obj.marked && !obj.freed;
}
fn is_dead(obj: GcObj) -> Bool {
  return !obj.marked || obj.freed;
}
fn reset_mark(obj: GcObj) -> GcObj {
  return GcObj{ id: obj.id; marked: false; reachable: false; size: obj.size; freed: false; };
}
fn mark_roots(a: GcObj, b: GcObj, c: GcObj) -> Int {
  var m = mark(a);
  if !m.marked { return 1; }
  return 0;
}
fn main() -> Int {
  var o1 = make_obj(1, 64);
  var o2 = make_obj(2, 128);
  var o3 = make_obj(3, 32);
  if o1.marked || o2.marked || o3.marked { return 1; }
  if is_live(o1) { return 2; }
  var m1 = mark(o1);
  if !is_live(m1) { return 3; }
  var m2 = mark(o2);
  if !is_live(m2) { return 4; }
  if is_live(o3) { return 5; }
  var s3 = sweep(o3);
  if s3.freed { return 6; }
  var r = reset_mark(m1);
  if r.marked { return 7; }
  if is_live(r) { return 8; }
  if !is_dead(o3) { return 9; }
  var mr = mark_roots(o1, o2, o3);
  if mr != 0 { return 10; }
  return 0;
}
