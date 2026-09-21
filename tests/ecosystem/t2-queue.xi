// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use xiom.io;
use xiom.sync.AtomicInt;
use xiom.math;

type SPSCQueue = {
  head: AtomicInt;
  tail: AtomicInt;
  capacity: Int;
  buffer: Vec[Int];
}

fn SPSCQueue.new(capacity: Int) -> SPSCQueue {
  var buf = Vec[Int].with_capacity(capacity + 1);
  var i: Int = 0;
  while i <= capacity {
    buf.push(0);
    i = i + 1;
  }
  return SPSCQueue{
    head: AtomicInt.new(0),
    tail: AtomicInt.new(0),
    capacity: capacity + 1,
    buffer: buf
  };
}

fn SPSCQueue.enqueue(val: Int) -> Bool {
  let current_tail: Int = tail.load();
  let next: Int = (current_tail + 1) % capacity;
  if next == head.load() {
    return false;
  }
  buffer[current_tail] = val;
  tail.store(next);
  return true;
}

fn SPSCQueue.dequeue() -> Option[Int] {
  let current_head: Int = head.load();
  if current_head == tail.load() {
    return None;
  }
  let val: Int = buffer[current_head];
  head.store((current_head + 1) % capacity);
  return Some(val);
}

fn main() -> Int {
  let cap: Int = 1000000;
  var q = SPSCQueue.new(cap);

  var i: Int = 0;
  while i < cap {
    q.enqueue(i);
    i = i + 1;
  }

  i = 0;
  while i < cap {
    q.dequeue();
    i = i + 1;
  }

  io.println("OK");
  return 0;
}
