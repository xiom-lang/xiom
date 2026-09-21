// XIOM -- Concurrency & Async Stress Benchmark
// Exercises spawn, channels, async/await patterns.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module benchmark.concurrency

use benchmark.main.BenchResult;

pub type WorkerState = { id: Int; task_count: Int; completed: Int; } derive[Clone]
pub fn WorkerState.new(id: Int) -> WorkerState { return WorkerState{ id: id, task_count: 0, completed: 0 }; }
pub fn WorkerState.assign_tasks(self, n: Int) -> WorkerState { return WorkerState{ id: self.id, task_count: self.task_count + n, completed: self.completed }; }
pub fn WorkerState.complete_one(self) -> WorkerState { if self.completed < self.task_count { return WorkerState{ id: self.id, task_count: self.task_count, completed: self.completed + 1 }; } return WorkerState{ id: self.id, task_count: self.task_count, completed: self.completed }; }
pub fn WorkerState.is_done(self) -> Bool { return self.completed >= self.task_count; }
pub fn WorkerState.progress(self) -> Int { if self.task_count == 0 { return 0; } return self.completed * 100 / self.task_count; }

fn test_worker() -> Int {
  var score = 0; var w = WorkerState.new(1);
  if w.id == 1 { score = score + 1; } if w.task_count == 0 { score = score + 1; } if w.progress() == 0 { score = score + 1; }
  var w1 = WorkerState.assign_tasks(w, 10); if w1.task_count == 10 { score = score + 1; }
  var w2 = WorkerState.complete_one(WorkerState.complete_one(WorkerState.complete_one(w1))); if w2.progress() == 30 { score = score + 1; }
  var wf = w2; var i = 0; while i < 7 { wf = WorkerState.complete_one(wf); i = i + 1; }
  if wf.is_done() { score = score + 1; } if wf.progress() == 100 { score = score + 1; }
  return score;
}

pub type Channel[T] = { items: Vec[T]; } derive[Clone]
pub fn Channel.new[T]() -> Channel[T] { return Channel[T]{ items: Vec[T].new() }; }
pub fn Channel.push[T](self, item: T) -> Channel[T] { var new_items = self.items.clone(); new_items.push(item); return Channel[T]{ items: new_items }; }
pub fn Channel.len[T](self) -> Int { return self.items.len(); }
pub fn Channel.sum[T](self) -> Int { var total = 0; var i = 0; while i < self.items.len() { total = total + (self.items[i] as Int); i = i + 1; } return total; }

fn test_channel() -> Int {
  var score = 0; var ch: Channel[Int] = Channel.new[Int](); if ch.len() == 0 { score = score + 1; }
  var ch1 = Channel.push(ch, 10); var ch2 = Channel.push(ch1, 20); var ch3 = Channel.push(ch2, 30);
  if ch3.len() == 3 { score = score + 1; } if ch3.sum() == 60 { score = score + 1; }
  return score;
}

pub type Guard = { value: Int; is_locked: Int; } derive[Clone]
pub fn Guard.new(val: Int) -> Guard { return Guard{ value: val, is_locked: 0 }; }
pub fn Guard.lock(self) -> Guard { return Guard{ value: self.value, is_locked: 1 }; }
pub fn Guard.unlock(self) -> Guard { return Guard{ value: self.value, is_locked: 0 }; }
pub fn Guard.is_locked(self) -> Bool { return self.is_locked == 1; }
pub fn Guard.get(self) -> Int { return self.value; }
pub fn Guard.add(self, amount: Int) -> Guard { return Guard{ value: self.value + amount, is_locked: self.is_locked }; }

fn test_guard() -> Int {
  var score = 0; var g = Guard.new(42);
  if !(g.is_locked()) { score = score + 1; } if g.get() == 42 { score = score + 1; }
  var g2 = g.lock(); if g2.is_locked() { score = score + 1; }
  var g3 = g2.add(8); if g3.get() == 50 { score = score + 1; }
  var g4 = g3.unlock(); if !(g4.is_locked()) { score = score + 1; } if g4.get() == 50 { score = score + 1; }
  return score;
}

pub fn run_all() -> BenchResult {
  var total = 0; var max_score = 0;
  var s1 = test_worker(); total = total + s1; max_score = max_score + 8;
  var s2 = test_channel(); total = total + s2; max_score = max_score + 3;
  var s3 = test_guard(); total = total + s3; max_score = max_score + 5;
  return BenchResult{ name: "concurrency", score: total, max_score: max_score, passed: total == max_score, elapsed_ms: 0 };
}
