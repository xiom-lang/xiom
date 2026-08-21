// M34-N2-06: 6-level nested struct S1 -> S2 -> S3 -> S4 -> S5 -> S6
type S1 = { val: Int; }
type S2 = { s1: S1; }
type S3 = { s2: S2; }
type S4 = { s3: S3; }
type S5 = { s4: S4; }
type S6 = { s5: S5; }

fn main() -> Int {
  var s = S6{ s5: S5{ s4: S4{ s3: S3{ s2: S2{ s1: S1{ val: 99; }; }; }; }; }; };
  if s.s5.s4.s3.s2.s1.val == 99 { return 0; }
  return 1;
}
