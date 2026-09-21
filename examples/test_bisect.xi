// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module data_primes {
  pub fn primes_table() -> Vec[Int] {
    var p = Vec[Int].new();
    return p;
  }
}

module types {


  // ============================================================
  // SECTION 1: Simple Structs & Field Access
  // ============================================================

  pub type Point2D = {
    x: Float64;
    y: Float64;
  } derive[Eq, Clone]

  pub type Point3D = {
    x: Float64;
    y: Float64;
    z: Float64;
  } derive[Eq, Clone]

  pub type Color = {
    r: Int;
    g: Int;
    b: Int;
    a: Int;
  } derive[Eq, Clone]

  pub type Size = {
    w: Int;
    h: Int;
  } derive[Eq, Clone]

  pub type Rect = {
    x: Int;
    y: Int;
    w: Int;
    h: Int;
  } derive[Eq, Clone]

  pub fn Point2D.new(x: Float64, y: Float64) -> Point2D {
    return Point2D{ x: x, y: y };
  }

  pub fn Point2D.dist_sq(other: &Point2D) -> Float64 {
    var dx = x - other.x;
    var dy = y - other.y;
    return dx * dx + dy * dy;
  }

  pub fn Point3D.new(x: Float64, y: Float64, z: Float64) -> Point3D {
    return Point3D{ x: x, y: y, z: z };
  }

  pub fn Point3D.dist_sq(other: &Point3D) -> Float64 {
    var dx = x - other.x;
    var dy = y - other.y;
    var dz = z - other.z;
    return dx * dx + dy * dy + dz * dz;
  }

  fn test_basic_structs() -> Int {
    var score = 0;
    var p1 = Point2D.new(0.0, 0.0);
    var p2 = Point2D.new(3.0, 4.0);

    if p1.x == 0.0 { score = score + 1; }
    if p1.y == 0.0 { score = score + 1; }
    if p2.x == 3.0 { score = score + 1; }
    if p2.y == 4.0 { score = score + 1; }
    if p1.dist_sq(&p2) == 25.0 { score = score + 1; }

    var p3 = Point3D.new(1.0, 2.0, 3.0);
    if p3.x == 1.0 { score = score + 1; }
    if p3.y == 2.0 { score = score + 1; }
    if p3.z == 3.0 { score = score + 1; }

    var p3b = Point3D.new(4.0, 6.0, 8.0);
    var dsq = p3.dist_sq(&p3b);
    if dsq == 50.0 { score = score + 1; }

    var c = Color{ r: 255, g: 128, b: 64, a: 255 };
    if c.r == 255 { score = score + 1; }
    if c.g == 128 { score = score + 1; }
    if c.b == 64 { score = score + 1; }
    if c.a == 255 { score = score + 1; }

    var s = Size{ w: 800, h: 600 };
    if s.w == 800 { score = score + 1; }
    if s.h == 600 { score = score + 1; }

    var r = Rect{ x: 10, y: 20, w: 100, h: 50 };
    if r.x == 10 { score = score + 1; }
    if r.y == 20 { score = score + 1; }
    if r.w == 100 { score = score + 1; }
    if r.h == 50 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 2: Nested Structs
  // ============================================================

  pub type Address = {
    street: Str;
    city: Str;
    zip: Int;
  } derive[Clone]

  pub type Person = {
    name: Str;
    age: Int;
    address: Address;
  } derive[Clone]

  pub type Company = {
    name: Str;
    employees: Int;
    hq: Address;
  } derive[Clone]

  pub type Transform = {
    pos: Point3D;
    rot: Point3D;
    scale: Point3D;
  } derive[Clone]

  fn test_nested_structs() -> Int {
    var score = 0;
    var addr = Address{ street: "Main", city: "NYC", zip: 10001 };
    if addr.zip == 10001 { score = score + 1; }

    var person = Person{ name: "Alice", age: 30, address: addr };
    if person.name == "Alice" { score = score + 1; }
    if person.age == 30 { score = score + 1; }
    if person.address.zip == 10001 { score = score + 1; }

    var company = Company{
      name: "ACME",
      employees: 500,
      hq: addr,
    };
    if company.name == "ACME" { score = score + 1; }
    if company.employees == 500 { score = score + 1; }
    if company.hq.city == "NYC" { score = score + 1; }

    var t = Transform{
      pos: Point3D.new(0.0, 0.0, 0.0),
      rot: Point3D.new(1.0, 0.0, 0.0),
      scale: Point3D.new(2.0, 2.0, 2.0),
    };
    if t.pos.x == 0.0 { score = score + 1; }
    if t.rot.x == 1.0 { score = score + 1; }
    if t.scale.x == 2.0 { score = score + 1; }

    return score;
  }

  // ============================================================
  // SECTION 3: Large Struct (50 Fields)
  // ============================================================

  pub type BigStruct = {
    f00: Int; f01: Int; f02: Int; f03: Int; f04: Int;
    f05: Int; f06: Int; f07: Int; f08: Int; f09: Int;
    f10: Int; f11: Int; f12: Int; f13: Int; f14: Int;
    f15: Int; f16: Int; f17: Int; f18: Int; f19: Int;
    f20: Int; f21: Int; f22: Int; f23: Int; f24: Int;
    f25: Int; f26: Int; f27: Int; f28: Int; f29: Int;
    f30: Int; f31: Int; f32: Int; f33: Int; f34: Int;
    f35: Int; f36: Int; f37: Int; f38: Int; f39: Int;
    f40: Int; f41: Int; f42: Int; f43: Int; f44: Int;
    f45: Int; f46: Int; f47: Int; f48: Int; f49: Int;
  } derive[Clone]

  pub fn BigStruct.new(val: Int) -> BigStruct {
    return BigStruct{
      f00: val, f01: val + 1, f02: val + 2, f03: val + 3, f04: val + 4,
      f05: val + 5, f06: val + 6, f07: val + 7, f08: val + 8, f09: val + 9,
      f10: val + 10, f11: val + 11, f12: val + 12, f13: val + 13, f14: val + 14,
      f15: val + 15, f16: val + 16, f17: val + 17, f18: val + 18, f19: val + 19,
      f20: val + 20, f21: val + 21, f22: val + 22, f23: val + 23, f24: val + 24,
      f25: val + 25, f26: val + 26, f27: val + 27, f28: val + 28, f29: val + 29,
      f30: val + 30, f31: val + 31, f32: val + 32, f33: val + 33, f34: val + 34,
      f35: val + 35, f36: val + 36, f37: val + 37, f38: val + 38, f39: val + 39,
      f40: val + 40, f41: val + 41, f42: val + 42, f43: val + 43, f44: val + 44,
      f45: val + 45, f46: val + 46, f47: val + 47, f48: val + 48, f49: val + 49,
    };
  }

  pub fn BigStruct.sum() -> Int {
    return f00 + f01 + f02 + f03 + f04 + f05 + f06 + f07 + f08 + f09
      + f10 + f11 + f12 + f13 + f14 + f15 + f16 + f17 + f18 + f19
      + f20 + f21 + f22 + f23 + f24 + f25 + f26 + f27 + f28 + f29
      + f30 + f31 + f32 + f33 + f34 + f35 + f36 + f37 + f38 + f39
      + f40 + f41 + f42 + f43 + f44 + f45 + f46 + f47 + f48 + f49;
  }

  // ============================================================
  // SECTION 4: Type Aliases & Composition
  // ============================================================

  pub type Vec2 = Point2D;
  pub type Vec3 = Point3D;

  pub type Matrix2x2 = {
    a11: Float64; a12: Float64;
    a21: Float64; a22: Float64;
  } derive[Clone]

  pub type Matrix3x3 = {
    m11: Float64; m12: Float64; m13: Float64;
    m21: Float64; m22: Float64; m23: Float64;
    m31: Float64; m32: Float64; m33: Float64;
  } derive[Clone]

  pub type Vec2 = Point2D;
  pub type Vec3 = Point3D;

  pub type Matrix2x2 = {
    a11: Float64; a12: Float64;
    a21: Float64; a22: Float64;
  } derive[Clone]

  pub type Matrix3x3 = {
    m11: Float64; m12: Float64; m13: Float64;
    m21: Float64; m22: Float64; m23: Float64;
    m31: Float64; m32: Float64; m33: Float64;
  } derive[Clone]
}
