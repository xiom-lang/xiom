// M36-S25: Reference counting pattern -- increment/decrement and release
type RcObj = { id: Int; ref_count: Int; data: Int; dropped: Bool; }
fn make_rc(id: Int, data: Int) -> RcObj {
  return RcObj{ id: id; ref_count: 1; data: data; dropped: false; };
}
fn retain(obj: RcObj) -> RcObj {
  return RcObj{ id: obj.id; ref_count: obj.ref_count + 1; data: obj.data; dropped: obj.dropped; };
}
fn release(obj: RcObj) -> RcObj {
  var new_count = obj.ref_count - 1;
  var dropped = false;
  if new_count <= 0 { dropped = true; new_count = 0; }
  return RcObj{ id: obj.id; ref_count: new_count; data: obj.data; dropped: dropped; };
}
fn is_alive(obj: RcObj) -> Bool {
  return obj.ref_count > 0 && !obj.dropped;
}
fn refs(obj: RcObj) -> Int {
  return obj.ref_count;
}
fn should_free(obj: RcObj) -> Bool {
  return obj.dropped || obj.ref_count <= 0;
}
fn main() -> Int {
  var rc = make_rc(1, 42);
  if refs(rc) != 1 { return 1; }
  if !is_alive(rc) { return 2; }
  var rc2 = retain(rc);
  if refs(rc2) != 2 { return 3; }
  var rc3 = retain(rc2);
  if refs(rc3) != 3 { return 4; }
  var rc4 = release(rc3);
  if refs(rc4) != 2 { return 5; }
  if rc4.dropped { return 6; }
  var rc5 = release(rc4);
  if refs(rc5) != 1 { return 7; }
  var rc6 = release(rc5);
  if refs(rc6) != 0 { return 8; }
  if !rc6.dropped { return 9; }
  if !should_free(rc6) { return 10; }
  if is_alive(rc6) { return 11; }
  return 0;
}
