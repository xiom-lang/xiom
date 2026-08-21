// M34-N2-04: 5-level nested generic type chain L1[T] -> L2[T] -> L3[T] -> L4[T] -> L5[T]
type L1[T] = { val: T; }
type L2[T] = { val: L1[T]; }
type L3[T] = { val: L2[T]; }
type L4[T] = { val: L3[T]; }
type L5[T] = { val: L4[T]; }

fn get_val[T](x: L5[T]) -> T { return x.val.val.val.val.val; }

fn main() -> Int {
  var v = L5[Int]{ val: L4[Int]{ val: L3[Int]{ val: L2[Int]{ val: L1[Int]{ val: 42; }; }; }; }; };
  var r = get_val[Int](v);
  if r == 42 { return 0; }
  return 1;
}
