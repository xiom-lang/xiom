// m88 (R46/R46b): same-leaf GENERIC types with conflicting shapes across
// project modules AND direct cross-module generic-method calls. Pre-R46b the
// qualified receiver `g.Box` resolved to a same-leaf sibling module in
// HashMap order (~25% of runs), so `Box.new`/`Box.pack` fell to erased
// auto-stubs returning zeroinitializer. The direct forms below are the lock:
// no module-local wrappers, explicit type args, arg inference, receiver-only
// inference, and a computed receiver.
module m88.main

use m88.generics as g;
use m88.hard as h;

fn main() -> Int {
  // Qualified receiver + explicit type arg + method with explicit type arg.
  var plain = g.Box.new[Int](7);
  if plain.value_of[Int]() != 7 { return 1; }

  // Arg-inferred instantiation + receiver-only method type-arg inference.
  var plain2 = g.Box.new(9);
  if plain2.value_of() != 9 { return 2; }

  // Same-leaf sibling module (2-field Box) on the same direct forms.
  var packed = h.Box.pack[Int](42);
  if packed.item_of[Int]() != 42 { return 3; }
  var packed2 = h.Box.pack(43);
  if packed2.item_of() != 43 { return 4; }
  if !packed2.is_sealed() { return 5; }

  // Computed receiver: the type arg comes from the receiver call's own
  // instantiation (`new[Str]` -> value_of must mono as Str, not the i64
  // "Int" fallback / literal-0 stub).
  if g.Box.new[Str]("tmp").value_of() != "tmp" { return 6; }

  return 0;
}
