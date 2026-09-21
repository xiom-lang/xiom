// m116 (R7 residual): a generic constructor's Holder[V] built with
// `Vec[V].new()`, then an aggregate element read through the holder's FIELD
// (`h.values[0]`). The binding lost the concrete args ("Holder[JsonValue]"
// was never recorded for local explicit-generic calls), so the index read
// fell to the scalar i64 switch and loaded the first 8 bytes of the inline
// aggregate as a pointer (0xC0000005 in p_generic_push/p_gp_b/p_gp_c).
module m116.generic_ctor_field_index

type Holder[V] = { values: Vec[V]; }

type Big = { a: Int; b: Int; c: Int; d: Int; }

fn make_holder[V]() -> Holder[V] {
  return Holder[V]{ values: Vec[V].new() };
}

fn add_h[V](h: &mut Holder[V], x: V) {
  h.values.push(x);
}

fn main() -> Int {
  // generic field push (method position)
  var h = make_holder[Big]();
  add_h(&h, Big{ a: 7, b: 8, c: 9, d: 10 });
  if h.values[0].a != 7 { return 1; }
  if h.values[0].d != 10 { return 2; }
  if h.values.len() != 1 { return 3; }

  // concrete push from main into the generic holder
  var h2 = make_holder[Big]();
  h2.values.push(Big{ a: 1, b: 2, c: 3, d: 4 });
  if h2.values[0].c != 3 { return 4; }
  return 0;
}
