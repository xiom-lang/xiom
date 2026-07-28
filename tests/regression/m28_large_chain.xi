// M28: Large program stress — 80 functions, 80 consts, mixed types
const C0: Int = 0;
const C1: Int = 1;
fn f0() -> Int { return C0; }
fn f1() -> Int { return C1 + f0(); }
fn f2() -> Int { return 2 + f1(); }
fn f3() -> Int { return 3 + f2(); }
fn f4() -> Int { return 4 + f3(); }
fn f5() -> Int { return 5 + f4(); }
fn f6() -> Int { return 6 + f5(); }
fn f7() -> Int { return 7 + f6(); }
fn f8() -> Int { return 8 + f7(); }
fn f9() -> Int { return 9 + f8(); }
fn f10() -> Int { return 10 + f9(); }
fn f11() -> Int { return 11 + f10(); }
fn f12() -> Int { return 12 + f11(); }
fn f13() -> Int { return 13 + f12(); }
fn f14() -> Int { return 14 + f13(); }
fn f15() -> Int { return 15 + f14(); }
fn f16() -> Int { return 16 + f15(); }
fn f17() -> Int { return 17 + f16(); }
fn f18() -> Int { return 18 + f17(); }
fn f19() -> Int { return 19 + f18(); }
fn f20() -> Int { return 20 + f19(); }
fn f21() -> Int { return 21 + f20(); }
fn f22() -> Int { return 22 + f21(); }
fn f23() -> Int { return 23 + f22(); }
fn f24() -> Int { return 24 + f23(); }
fn f25() -> Int { return 25 + f24(); }
fn f26() -> Int { return 26 + f25(); }
fn f27() -> Int { return 27 + f26(); }
fn f28() -> Int { return 28 + f27(); }
fn f29() -> Int { return 29 + f28(); }
fn f30() -> Int { return 30 + f29(); }
fn f31() -> Int { return 31 + f30(); }
fn f32() -> Int { return 32 + f31(); }
fn f33() -> Int { return 33 + f32(); }
fn f34() -> Int { return 34 + f33(); }
fn f35() -> Int { return 35 + f34(); }
fn f36() -> Int { return 36 + f35(); }
fn f37() -> Int { return 37 + f36(); }
fn f38() -> Int { return 38 + f37(); }
fn f39() -> Int { return 39 + f38(); }
// Sum of 0..39 = 39*40/2 = 780
fn main() -> Int {
  if f39() == 780 { return 0; }
  return 1;
}
