// m368 lock (REL-20261011-2032-engine): an owned Vec local whose only uses
// are non-escaping (push/len) must free its data buffer at function exit.
// Pre-fix `alloc_one` malloc/realloc'd ~8 MB per call and freed nothing, so
// the engine's alloc+drop loop grew RSS ~1 GB/s (15.9 GB / 10 s at v0.64.4).
// The escape controls below must KEEP leaking (never free): `make_vec`
// returns its Vec, `alias_control` binds a copy of another owned Vec.
// Pure XIOM -- no C source. Copyright (c) 2026 Eleftherios Notas and The XIOM
// Authors -- SPDX-License-Identifier: MIT OR Apache-2.0
module m368_vec_drop_reclaim

use xiom.io;

fn alloc_one(n: Int) -> Int {
    var v = Vec[Float64].new();
    var j = 0;
    while j < n {
        v.push(1.0);
        j = j + 1;
    }
    return v.len();
}

fn alloc_let(n: Int) -> Int {
    let v = Vec[Int].new();
    var j = 0;
    while j < n {
        v.push(j);
        j = j + 1;
    }
    return v.len();
}

fn make_vec() -> Vec[Int] {
    var v = Vec[Int].new();
    v.push(1);
    v.push(2);
    return v;
}

fn alias_control() -> Int {
    var a = Vec[Int].new();
    a.push(7);
    var b = a;
    b.push(8);
    return a.len() * 10 + b.len();
}

fn main() -> Int {
    var iter = 0;
    while iter < 64 {
        var n = alloc_one(100000);
        if n != 100000 { return 1; }
        var m = alloc_let(1000);
        if m != 1000 { return 2; }
        iter = iter + 1;
    }

    var w = make_vec();
    if w.len() != 2 { return 3; }
    w.push(3);
    if w[2] != 3 { return 4; }

    var c = alias_control();
    if c != 12 { return 5; }

    println("vec drop reclaim ok");
    return 0;
}
