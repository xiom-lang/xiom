// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m318 lock (REL-20261013-2215-engine): a struct-returning fn whose body is a
// confined (whole-body `unsafe`) block and whose match arms push onto a Vec
// returned a struct whose inline Vec fields pointed at the guard arena. The
// exit released the arena, so the caller's first element read was silent
// garbage / STATUS_ACCESS_VIOLATION 0xC0000005 with no diagnostic (the engine
// world/state.xi state-machine workaround). m309 promoted only a DIRECT
// `%struct.Vec` tail; aggregates were left dangling. Each shape below returns
// through a different lowering path (return-inside-block, block tail, local
// Ident tail, depth-2 nested struct field) and is exercised with exact values
// PLUS a post-return push proving the promoted buffer is a live main-heap
// allocation. Controls: an i64-only aggregate (no Vec) and a non-confined fn.
module m318_vec_match_arm_confined;

use xiom.io;

// Engine-shaped state aggregate: inline Vec field FIRST.
pub type World = {
  cells: Vec[Int];
  epoch: Int;
}

// Depth-2 aggregate: the Vec sits inside a nested struct field.
pub type Grid = {
  marks: Vec[Int];
  tag: Int;
}

pub type Layered = {
  grid: Grid;
  epoch: Int;
}

// Control: aggregate with no container fields.
pub type Counter = {
  n: Int;
  m: Int;
}

// A: `return World{...}` INSIDE the confined block (stmt.rs return path);
// the Vec is created inside the block, so its buffer routes to the arena.
fn step_return(cmd: Int) -> World
  requires: true
{
  unsafe {
    var cells: Vec[Int] = Vec[Int].new();
    var epoch: Int = 0;
    match cmd {
      0 => {
        cells.push(10);
        cells.push(11);
        epoch = 1;
      }
      1 => {
        cells.push(20);
        epoch = 2;
      }
      _ => {
        epoch = 9;
      }
    }
    return World { cells: cells; epoch: epoch };
  }
}

// B: the struct literal is the confined block's TAIL (expr.rs tail path).
fn step_tail(cmd: Int) -> World
  requires: true
{
  unsafe {
    var cells: Vec[Int] = Vec[Int].new();
    var epoch: Int = 0;
    match cmd {
      0 => {
        cells.push(30);
        epoch = 1;
      }
      _ => {
        epoch = 9;
      }
    }
    World { cells: cells; epoch: epoch }
  }
}

// C: aggregate tail is a local Ident whose field was mutated through the arms.
fn step_local(cmd: Int) -> World
  requires: true
{
  unsafe {
    var w = World { cells: Vec[Int].new(); epoch: 0 };
    match cmd {
      0 => {
        w.cells.push(40);
        w.epoch = 1;
      }
      _ => {
        w.epoch = 9;
      }
    }
    return w;
  }
}

// D: depth-2 aggregate (Vec inside the nested Grid field).
fn step_layered(mark: Int) -> Layered
  requires: true
{
  unsafe {
    var grid = Grid { marks: Vec[Int].new(); tag: 0 };
    match mark {
      0 => {
        grid.marks.push(50);
        grid.tag = 1;
      }
      _ => {
        grid.tag = 9;
      }
    }
    return Layered { grid: grid; epoch: 5 };
  }
}

// G: the literal engine shape -- `vec.push(...)` is the TAIL of a confined
// statement block inside a match arm of a struct-returning fn. The block's
// value is the push result (a %struct.Vec) while the fn returns World: the
// pre-fix block site reconstructed the trampoline result with the block tail
// type and emitted `ret %struct.World %<loaded %struct.Vec>`, which clang
// rejects ("defined with type ... but expected ..."). No return exists in the
// block, so the was_returned branch is gated away entirely.
fn step_push_arm(cmd: Int) -> World {
  var items: Vec[Int] = Vec[Int].new();
  var epoch: Int = 0;
  match cmd {
    0 => {
      unsafe {
        epoch = 1;
        items.push(70);
      }
    }
    _ => {
      epoch = 9;
    }
  }
  return World { cells: items; epoch: epoch };
}

// Control E: i64-only aggregate out of a confined block (no Copy-Out needed).
fn count_only(k: Int) -> Counter
  requires: true
{
  unsafe {
    var c = Counter { n: k; m: 0 };
    match k {
      0 => {
        c.m = 1;
      }
      _ => {
        c.m = 9;
      }
    }
    return c;
  }
}

// Control F: non-confined struct-returning fn with push in match arms -- the
// legacy all-green shape on every baseline.
fn plain_step(cmd: Int) -> World {
  var cells: Vec[Int] = Vec[Int].new();
  var epoch: Int = 0;
  match cmd {
    0 => {
      cells.push(60);
      epoch = 1;
    }
    _ => {
      epoch = 9;
    }
  }
  return World { cells: cells; epoch: epoch };
}

fn main() -> Int {
  // A: return-inside-block, two elements.
  var a = step_return(0);
  io.println("A: epoch=" + to_string(a.epoch) + " len=" + to_string(a.cells.len()));
  if a.epoch != 1 { return 1; }
  if a.cells.len() != 2 { return 2; }
  if a.cells[0] != 10 { return 3; }
  if a.cells[1] != 11 { return 4; }
  // Post-return mutation proves the promoted buffer is live main-heap memory.
  a.cells.push(12);
  if a.cells.len() != 3 { return 5; }
  if a.cells[2] != 12 { return 6; }
  var a1 = step_return(1);
  if a1.epoch != 2 { return 7; }
  if a1.cells.len() != 1 { return 8; }
  if a1.cells[0] != 20 { return 9; }

  // B: block-tail form.
  var b = step_tail(0);
  io.println("B: epoch=" + to_string(b.epoch) + " len=" + to_string(b.cells.len()));
  if b.epoch != 1 { return 10; }
  if b.cells.len() != 1 { return 11; }
  if b.cells[0] != 30 { return 12; }
  b.cells.push(31);
  if b.cells.len() != 2 { return 13; }
  if b.cells[1] != 31 { return 14; }

  // C: local Ident tail.
  var c = step_local(0);
  io.println("C: epoch=" + to_string(c.epoch) + " len=" + to_string(c.cells.len()));
  if c.epoch != 1 { return 15; }
  if c.cells.len() != 1 { return 16; }
  if c.cells[0] != 40 { return 17; }
  c.cells.push(41);
  if c.cells.len() != 2 { return 18; }
  if c.cells[1] != 41 { return 19; }

  // D: depth-2 nested grid.
  var d = step_layered(0);
  io.println("D: tag=" + to_string(d.grid.tag) + " epoch=" + to_string(d.epoch)
    + " len=" + to_string(d.grid.marks.len()));
  if d.grid.tag != 1 { return 20; }
  if d.epoch != 5 { return 21; }
  if d.grid.marks.len() != 1 { return 22; }
  if d.grid.marks[0] != 50 { return 23; }
  d.grid.marks.push(51);
  if d.grid.marks.len() != 2 { return 24; }
  if d.grid.marks[1] != 51 { return 25; }

  // E: i64-only control.
  var e = count_only(0);
  if e.n != 0 { return 26; }
  if e.m != 1 { return 27; }

  // F: non-confined control.
  var f = plain_step(0);
  if f.epoch != 1 { return 28; }
  if f.cells.len() != 1 { return 29; }
  if f.cells[0] != 60 { return 30; }

  // G: push-tail in a confined arm (the loud pre-fix clang reject; here a
  // value check too -- the vec itself lives outside the block and stays fine).
  var g = step_push_arm(0);
  io.println("G: epoch=" + to_string(g.epoch) + " len=" + to_string(g.cells.len()));
  if g.epoch != 1 { return 31; }
  if g.cells.len() != 1 { return 32; }
  if g.cells[0] != 70 { return 33; }
  var g1 = step_push_arm(1);
  if g1.epoch != 9 { return 34; }
  if g1.cells.len() != 0 { return 35; }

  io.println("m318 vec-in-match-arm confined aggregate: all arms green");
  return 0;
}
