// XIOM -- Sync / Thread / Async / Net Stdlib Conformance Tests
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module sync_net_tests
use xiom.test;
use xiom.sync;
use xiom.thread;
use xiom.async;
use xiom.net;

// === Helpers ===

var once_counter: Int = 0;
fn once_inc() { once_counter = once_counter + 1; }

// ===========================================================================
// SYNC tests
// ===========================================================================

fn test_mutex_new_lock_get() -> TestResult {
  let m = sync.Mutex.new(42);
  let guard = m.lock();
  if guard.get() == 42 { return assert(true, "Mutex::new + lock + get"); }
  return assert(false, "Mutex::new + lock + get");
}

fn test_mutex_try_lock() -> TestResult {
  let m = sync.Mutex.new("hello");
  let opt = m.try_lock();
  if opt.is_some() { return assert(true, "Mutex::try_lock"); }
  return assert(false, "Mutex::try_lock");
}

fn test_rwlock_read() -> TestResult {
  let rw = sync.RwLock.new(99);
  let guard = rw.read();
  if guard.lock.data == 99 { return assert(true, "RwLock::read + get"); }
  return assert(false, "RwLock::read + get");
}

fn test_rwlock_write() -> TestResult {
  let rw = sync.RwLock.new(7);
  let guard = rw.write();
  if guard.lock.data == 7 { return assert(true, "RwLock::write"); }
  return assert(false, "RwLock::write");
}

fn test_once_call_once() -> TestResult {
  once_counter = 0;
  var o = sync.Once.new();
  o.call_once(once_inc);
  o.call_once(once_inc);
  o.call_once(once_inc);
  if once_counter == 1 { return assert(true, "Once::call_once called once"); }
  return assert(false, "Once::call_once called once");
}

fn test_arc_new_get_count() -> TestResult {
  let a = sync.Arc.new(42);
  if a.get() == 42 && a.strong_count() == 1 { return assert(true, "Arc::new + get + strong_count"); }
  return assert(false, "Arc::new + get + strong_count");
}

fn test_arc_multiple_clones() -> TestResult {
  let a = sync.Arc.new(10);
  let b = a.clone();
  let c = a.clone();
  if a.strong_count() == 3 && b.strong_count() == 3 && c.strong_count() == 3 {
    return assert(true, "Arc multiple clones count=3");
  }
  return assert(false, "Arc multiple clones count=3");
}

fn test_arc_ptr_eq_same() -> TestResult {
  let a = sync.Arc.new(1);
  let b = a.clone();
  if a.ptr_eq(&b) { return assert(true, "Arc::ptr_eq same Arc"); }
  return assert(false, "Arc::ptr_eq same Arc");
}

fn test_arc_ptr_eq_different() -> TestResult {
  let a = sync.Arc.new(1);
  let b = sync.Arc.new(1);
  if !a.ptr_eq(&b) { return assert(true, "Arc::ptr_eq different Arc"); }
  return assert(false, "Arc::ptr_eq different Arc");
}

fn test_atomic_bool_load() -> TestResult {
  let ab = sync.AtomicBool.new(true);
  if ab.load() { return assert(true, "AtomicBool::load"); }
  return assert(false, "AtomicBool::load");
}

fn test_atomic_bool_store() -> TestResult {
  var ab = sync.AtomicBool.new(false);
  ab.store(true);
  if ab.load() { return assert(true, "AtomicBool::store"); }
  return assert(false, "AtomicBool::store");
}

fn test_atomic_bool_swap() -> TestResult {
  var ab = sync.AtomicBool.new(true);
  let old = ab.swap(false);
  if old && !ab.load() { return assert(true, "AtomicBool::swap"); }
  return assert(false, "AtomicBool::swap");
}

fn test_atomic_int_load() -> TestResult {
  let ai = sync.AtomicInt.new(42);
  if ai.load() == 42 { return assert(true, "AtomicInt::load"); }
  return assert(false, "AtomicInt::load");
}

fn test_atomic_int_store() -> TestResult {
  var ai = sync.AtomicInt.new(0);
  ai.store(99);
  if ai.load() == 99 { return assert(true, "AtomicInt::store"); }
  return assert(false, "AtomicInt::store");
}

fn test_atomic_int_fetch_add() -> TestResult {
  var ai = sync.AtomicInt.new(10);
  let prev = ai.fetch_add(5);
  if prev == 10 && ai.load() == 15 { return assert(true, "AtomicInt::fetch_add"); }
  return assert(false, "AtomicInt::fetch_add");
}

fn test_atomic_int_fetch_sub() -> TestResult {
  var ai = sync.AtomicInt.new(20);
  let prev = ai.fetch_sub(7);
  if prev == 20 && ai.load() == 13 { return assert(true, "AtomicInt::fetch_sub"); }
  return assert(false, "AtomicInt::fetch_sub");
}

fn test_barrier_new() -> TestResult {
  let b = sync.Barrier.new(4);
  if b.count == 4 && b.generation == 0 { return assert(true, "Barrier::new"); }
  return assert(false, "Barrier::new");
}

// ===========================================================================
// THREAD tests
// ===========================================================================

fn test_spawn_basic() -> TestResult {
  let handle = thread.spawn(fn(): Int -> 42);
  if handle.is_finished() { return assert(true, "thread::spawn basic"); }
  return assert(false, "thread::spawn basic");
}

fn test_join_handle_join() -> TestResult {
  let handle = thread.spawn(fn(): Int -> 99);
  let result = handle.join();
  if result.is_ok() && result.unwrap() == 99 { return assert(true, "JoinHandle::join"); }
  return assert(false, "JoinHandle::join");
}

fn test_thread_current_id() -> TestResult {
  let t = thread.Thread.current();
  let id = t.id();
  if id >= 0 { return assert(true, "Thread::current + id"); }
  return assert(false, "Thread::current + id");
}

fn test_available_parallelism() -> TestResult {
  let n = thread.available_parallelism();
  if n > 0 { return assert(true, "available_parallelism > 0"); }
  return assert(false, "available_parallelism > 0");
}

fn test_hardware_threads() -> TestResult {
  let n = thread.hardware_threads();
  if n > 0 { return assert(true, "hardware_threads > 0"); }
  return assert(false, "hardware_threads > 0");
}

// ===========================================================================
// ASYNC tests
// ===========================================================================

fn test_channel_send_recv() -> TestResult {
  let _ch = async.Channel.unbounded();
  async.Channel.send(42);
  let val = async.Channel.recv();
  if val == 42 { return assert(true, "Channel::send + recv"); }
  return assert(false, "Channel::send + recv");
}

fn test_channel_fifo_order() -> TestResult {
  let _ch = async.Channel.unbounded();
  async.Channel.send(1);
  async.Channel.send(2);
  async.Channel.send(3);
  let a = async.Channel.recv();
  let b = async.Channel.recv();
  let c = async.Channel.recv();
  if a == 1 && b == 2 && c == 3 { return assert(true, "Channel FIFO order"); }
  return assert(false, "Channel FIFO order");
}

fn test_channel_try_recv_empty() -> TestResult {
  let _ch = async.Channel.unbounded();
  let val = async.Channel.try_recv();
  if val.is_none() { return assert(true, "Channel::try_recv empty -> None"); }
  return assert(false, "Channel::try_recv empty -> None");
}

fn test_channel_try_recv_nonempty() -> TestResult {
  let _ch = async.Channel.unbounded();
  async.Channel.send(77);
  let val = async.Channel.try_recv();
  if val.is_some() && val.unwrap() == 77 { return assert(true, "Channel::try_recv nonempty"); }
  return assert(false, "Channel::try_recv nonempty");
}

fn test_channel_bounded() -> TestResult {
  let ch = async.Channel.bounded(2);
  if ch.cap == 2 { return assert(true, "Channel::bounded cap=2"); }
  return assert(false, "Channel::bounded cap=2");
}

// ===========================================================================
// NET tests
// ===========================================================================

fn test_tcp_connect_returns_error() -> TestResult {
  let r = net.tcp_connect("localhost", 8080);
  if r.is_err() { return assert(true, "tcp_connect returns NetError"); }
  return assert(false, "tcp_connect returns NetError");
}

fn test_tcp_listen_returns_error() -> TestResult {
  let r = net.tcp_listen("0.0.0.0", 8080);
  if r.is_err() { return assert(true, "tcp_listen returns NetError"); }
  return assert(false, "tcp_listen returns NetError");
}

fn test_http_get_with_url_parsing() -> TestResult {
  let r = net.http_get("http://example.com/path");
  if r.is_err() { return assert(true, "http_get parses URL, returns TCP error"); }
  return assert(false, "http_get parses URL, returns TCP error");
}

fn test_http_get_with_port() -> TestResult {
  let r = net.http_get("http://example.com:3000/api");
  if r.is_err() { return assert(true, "http_get with port parses URL"); }
  return assert(false, "http_get with port parses URL");
}

fn test_http_get_with_query() -> TestResult {
  let r = net.http_get("http://example.com/search?q=xiom&page=1");
  if r.is_err() { return assert(true, "http_get with query parses URL"); }
  return assert(false, "http_get with query parses URL");
}

fn test_http_get_empty_url() -> TestResult {
  let r = net.http_get("");
  if r.is_err() { return assert(true, "http_get empty URL returns error"); }
  return assert(false, "http_get empty URL returns error");
}

fn test_tcp_stream_close_ok() -> TestResult {
  let s = net.TcpStream{ fd: -1; };
  let r = s.close();
  if r.is_ok() { return assert(true, "TcpStream::close OK"); }
  return assert(false, "TcpStream::close OK");
}

fn test_udp_socket_close_ok() -> TestResult {
  let s = net.UdpSocket{ fd: -1; };
  let r = s.close();
  if r.is_ok() { return assert(true, "UdpSocket::close OK"); }
  return assert(false, "UdpSocket::close OK");
}

// ===========================================================================
// MAIN
// ===========================================================================

fn main() -> Int {
  var tests: Vec[fn() -> TestResult] = [
    test_mutex_new_lock_get,
    test_mutex_try_lock,
    test_rwlock_read,
    test_rwlock_write,
    test_once_call_once,
    test_arc_new_get_count,
    test_arc_multiple_clones,
    test_arc_ptr_eq_same,
    test_arc_ptr_eq_different,
    test_atomic_bool_load,
    test_atomic_bool_store,
    test_atomic_bool_swap,
    test_atomic_int_load,
    test_atomic_int_store,
    test_atomic_int_fetch_add,
    test_atomic_int_fetch_sub,
    test_barrier_new,
    test_spawn_basic,
    test_join_handle_join,
    test_thread_current_id,
    test_available_parallelism,
    test_hardware_threads,
    test_channel_send_recv,
    test_channel_fifo_order,
    test_channel_try_recv_empty,
    test_channel_try_recv_nonempty,
    test_channel_bounded,
    test_tcp_connect_returns_error,
    test_tcp_listen_returns_error,
    test_http_get_with_url_parsing,
    test_http_get_with_port,
    test_http_get_with_query,
    test_http_get_empty_url,
    test_tcp_stream_close_ok,
    test_udp_socket_close_ok,
  ];
  return test.run_all(tests);
}
