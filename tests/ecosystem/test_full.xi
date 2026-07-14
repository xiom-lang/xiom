// XIOM — Ecosystem Full Feature Hardening Tests
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
//
// Self-contained integration test exercising enums with data-carrying
// variants, nested structs, state machines, concrete stack operations,
// match/while/if-elif-else, Result/Option with ? propagation,
// contracts (requires/ensures), and Vec push/pop.

module tests.ecosystem.test_full

// ============================================================================
// Types
// ============================================================================

pub enum AgentState {
  Idle,
  Running(task: Str, progress: Int),
  Waiting(reason: Str, retries: Int),
  Done(result: Str),
  Failed(error: Str),
}

pub type Config = {
  max_retries: Int;
  timeout_ms: Int;
  verbose: Bool;
}

pub type Agent = {
  state: AgentState;
  config: Config;
  task_count: Int;
}

pub type IntStack = {
  data: Vec[Int];
}

// ============================================================================
// Agent: Constructor
// ============================================================================

fn new_agent(config: Config) -> Agent {
  return Agent{
    state: AgentState.Idle,
    config: config,
    task_count: 0,
  };
}

// ============================================================================
// Agent: State predicates (exercises match on enum)
// ============================================================================

fn agent_is_idle(a: &Agent) -> Bool {
  match a.state {
    AgentState.Idle => { return true; }
    _ => { return false; }
  }
}

fn agent_is_running(a: &Agent) -> Bool {
  match a.state {
    AgentState.Running(_, _) => { return true; }
    _ => { return false; }
  }
}

fn agent_is_waiting(a: &Agent) -> Bool {
  match a.state {
    AgentState.Waiting(_, _) => { return true; }
    _ => { return false; }
  }
}

fn agent_is_done(a: &Agent) -> Bool {
  match a.state {
    AgentState.Done(_) => { return true; }
    _ => { return false; }
  }
}

fn agent_is_failed(a: &Agent) -> Bool {
  match a.state {
    AgentState.Failed(_) => { return true; }
    _ => { return false; }
  }
}

// ============================================================================
// Agent: State transitions (exercises Result and match)
// ============================================================================

fn agent_start(a: &mut Agent, task: Str) -> Result[Str, Str] {
  if !agent_is_idle(a) {
    return Err("agent is not idle");
  }
  a.state = AgentState.Running(task, 0);
  a.task_count = a.task_count + 1;
  return Ok("started: " + task);
}

fn agent_complete(a: &mut Agent, result: Str) -> Result[Str, Str] {
  match a.state {
    AgentState.Running(task, _) => {
      if a.config.verbose {
        a.state = AgentState.Done(result + " (from " + task + ")");
      } else {
        a.state = AgentState.Done(result);
      }
      return Ok("completed");
    }
    _ => { return Err("agent is not running"); }
  }
}

fn agent_fail(a: &mut Agent, error: Str) -> Result[Str, Str] {
  match a.state {
    AgentState.Running(task, _) => {
      a.state = AgentState.Failed("task '" + task + "' failed: " + error);
      return Ok("failed");
    }
    _ => { return Err("agent is not running"); }
  }
}

fn agent_wait(a: &mut Agent, reason: Str, retries: Int) -> Result[Str, Str] {
  if retries <= 0 {
    return Err("retries must be positive");
  }
  match a.state {
    AgentState.Running(task, _) => {
      a.state = AgentState.Waiting(reason + " (" + task + ")", retries);
      return Ok("waiting");
    }
    _ => { return Err("agent is not running"); }
  }
}

fn agent_retry(a: &mut Agent) -> Result[Str, Str] {
  match a.state {
    AgentState.Waiting(reason, retries) => {
      if retries <= 0 {
        a.state = AgentState.Failed("exhausted retries: " + reason);
        return Err("no retries remaining");
      }
      var remaining = retries - 1;
      a.state = AgentState.Running(reason, remaining);
      return Ok("retrying, " + int_to_str(remaining) + " left");
    }
    _ => { return Err("agent is not waiting"); }
  }
}

// ============================================================================
// Agent: Run a full state machine cycle
// ============================================================================

fn agent_run_cycle(a: &mut Agent, tasks: &Vec[Str]) -> Result[Str, Str] {
  var i = 0;
  var last = "";
  while i < tasks.len() {
    let start_r = agent_start(a, tasks[i]);
    if start_r.is_err() { return start_r; }
    last = tasks[i];

    var attempt = 0;
    var done = false;
    while attempt < a.config.max_retries && !done {
      var outcome = attempt % 3;
      if outcome == 0 {
        let r = agent_complete(a, last + " OK");
        if r.is_err() { return r; }
        done = true;
      } elif outcome == 1 {
        let r = agent_wait(a, "timeout", a.config.max_retries - attempt);
        if r.is_err() { return r; }
        let retry_r = agent_retry(a);
        if retry_r.is_err() { return retry_r; }
      } else {
        if attempt >= a.config.max_retries - 1 {
          let r = agent_fail(a, "max attempts reached");
          if r.is_err() { return r; }
          done = true;
        } else {
          let r = agent_wait(a, "error", 1);
          if r.is_err() { return r; }
          let retry_r = agent_retry(a);
          if retry_r.is_err() { return retry_r; }
        }
      }
      attempt = attempt + 1;
    }
    if !done {
      let r = agent_fail(a, "cycle incomplete");
      if r.is_err() { return r; }
    }
    i = i + 1;
  }
  return Ok("cycle complete: " + int_to_str(a.task_count) + " tasks");
}

// ============================================================================
// Int conversion helper
// ============================================================================

fn int_to_str(n: Int) -> Str {
  if n == 0 { return "0"; }
  var neg = false;
  var val = n;
  if val < 0 { neg = true; val = -val; }
  var buf = "";
  while val > 0 {
    var digit = val % 10;
    val = val / 10;
    var ch = "";
    if digit == 0 { ch = "0"; }
    elif digit == 1 { ch = "1"; }
    elif digit == 2 { ch = "2"; }
    elif digit == 3 { ch = "3"; }
    elif digit == 4 { ch = "4"; }
    elif digit == 5 { ch = "5"; }
    elif digit == 6 { ch = "6"; }
    elif digit == 7 { ch = "7"; }
    elif digit == 8 { ch = "8"; }
    elif digit == 9 { ch = "9"; }
    buf = ch + buf;
  }
  if neg { buf = "-" + buf; }
  return buf;
}

// ============================================================================
// IntStack: Concrete stack operations (exercises Vec push/pop, Option)
// ============================================================================

fn new_int_stack() -> IntStack {
  var data = Vec[Int].new();
  return IntStack{ data: data };
}

fn int_stack_push(s: &mut IntStack, val: Int) {
  s.data.push(val);
}

fn int_stack_pop(s: &mut IntStack) -> Option[Int] {
  if s.data.len() == 0 {
    return None;
  }
  var last = s.data[s.data.len() - 1];
  s.data.remove(s.data.len() - 1);
  return Some(last);
}

fn int_stack_peek(s: &IntStack) -> Option[Int] {
  if s.data.len() == 0 {
    return None;
  }
  return Some(s.data[s.data.len() - 1]);
}

fn int_stack_is_empty(s: &IntStack) -> Bool {
  return s.data.len() == 0;
}

fn int_stack_len(s: &IntStack) -> Int {
  return s.data.len();
}

// ============================================================================
// Standalone functions with contracts (exercises requires/ensures)
// ============================================================================

fn factorial(n: Int) -> Int
  requires: n >= 0;
  ensures: result >= 1;
{
  if n <= 1 { return 1; }
  var acc = 1;
  var i = 2;
  while i <= n {
    acc = acc * i;
    i = i + 1;
  }
  return acc;
}

fn safe_divide(a: Int, b: Int) -> Result[Int, Str]
  requires: b != 0;
{
  if b == 0 {
    return Err("division by zero");
  }
  return Ok(a / b);
}

fn isqrt(x: Int) -> Result[Int, Str]
  requires: x >= 0;
  ensures: result >= 0;
{
  if x < 0 {
    return Err("negative input");
  }
  if x <= 1 { return Ok(x); }
  var lo = 0;
  var hi = x;
  while lo <= hi {
    var mid = (lo + hi) / 2;
    var sq = mid * mid;
    if sq == x { return Ok(mid); }
    elif sq < x && sq >= 0 { lo = mid + 1; }
    else { hi = mid - 1; }
  }
  return Ok(hi);
}

// ============================================================================
// Functions exercising ? operator
// ============================================================================

fn chained_divide() -> Result[Int, Str] {
  let a = safe_divide(100, 5)?;
  let b = safe_divide(a, 2)?;
  let c = safe_divide(b, 2)?;
  return Ok(c);
}

fn propagate_zero_div() -> Result[Int, Str] {
  let x = safe_divide(10, 0)?;
  return Ok(x);
}

fn option_chain(val: Option[Int]) -> Option[Int] {
  if val.is_none() { return None; }
  var x = val.unwrap();
  if x <= 0 { return None; }
  return Some(x * 2);
}

// ============================================================================
// Tests
// ============================================================================

fn test_agent_new_is_idle() -> Bool {
  var cfg = Config{ max_retries: 3, timeout_ms: 5000, verbose: false };
  var a = new_agent(cfg);
  return agent_is_idle(&a);
}

fn test_agent_start_transition() -> Bool {
  var cfg = Config{ max_retries: 3, timeout_ms: 5000, verbose: false };
  var a = new_agent(cfg);
  let r = agent_start(&mut a, "deploy");
  return r.is_ok() && agent_is_running(&a) && a.task_count == 1;
}

fn test_agent_complete_transition() -> Bool {
  var cfg = Config{ max_retries: 3, timeout_ms: 5000, verbose: false };
  var a = new_agent(cfg);
  agent_start(&mut a, "test");
  let r = agent_complete(&mut a, "success");
  return r.is_ok() && agent_is_done(&a);
}

fn test_agent_fail_transition() -> Bool {
  var cfg = Config{ max_retries: 3, timeout_ms: 5000, verbose: false };
  var a = new_agent(cfg);
  agent_start(&mut a, "risky");
  let r = agent_fail(&mut a, "connection lost");
  return r.is_ok() && agent_is_failed(&a);
}

fn test_agent_wait_retry_cycle() -> Bool {
  var cfg = Config{ max_retries: 3, timeout_ms: 5000, verbose: false };
  var a = new_agent(cfg);
  agent_start(&mut a, "flaky");
  let w = agent_wait(&mut a, "network", 2);
  if w.is_err() { return false; }
  if !agent_is_waiting(&a) { return false; }
  let r = agent_retry(&mut a);
  return r.is_ok() && agent_is_running(&a);
}

fn test_agent_wait_zero_retries_rejected() -> Bool {
  var cfg = Config{ max_retries: 3, timeout_ms: 5000, verbose: false };
  var a = new_agent(cfg);
  agent_start(&mut a, "doomed");
  let r = agent_wait(&mut a, "oops", 0);
  return r.is_err();
}

fn test_agent_start_requires_idle() -> Bool {
  var cfg = Config{ max_retries: 3, timeout_ms: 5000, verbose: false };
  var a = new_agent(cfg);
  agent_start(&mut a, "first");
  let r = agent_start(&mut a, "second");
  return r.is_err();
}

fn test_agent_run_cycle() -> Bool {
  var cfg = Config{ max_retries: 2, timeout_ms: 1000, verbose: false };
  var a = new_agent(cfg);
  var tasks = Vec[Str].new();
  tasks.push("init");
  tasks.push("build");
  tasks.push("deploy");
  let r = agent_run_cycle(&mut a, &tasks);
  return r.is_ok();
}

fn test_stack_push_pop_lifo() -> Bool {
  var s = new_int_stack();
  int_stack_push(&mut s, 10);
  int_stack_push(&mut s, 20);
  int_stack_push(&mut s, 30);
  let a = int_stack_pop(&mut s);
  let b = int_stack_pop(&mut s);
  let c = int_stack_pop(&mut s);
  if a.is_none() || b.is_none() || c.is_none() { return false; }
  return a.unwrap() == 30 && b.unwrap() == 20 && c.unwrap() == 10;
}

fn test_stack_pop_empty() -> Bool {
  var s = new_int_stack();
  let val = int_stack_pop(&mut s);
  return val.is_none();
}

fn test_stack_peek_does_not_remove() -> Bool {
  var s = new_int_stack();
  int_stack_push(&mut s, 42);
  let p = int_stack_peek(&s);
  if p.is_none() { return false; }
  if p.unwrap() != 42 { return false; }
  return int_stack_len(&s) == 1;
}

fn test_stack_is_empty_and_len() -> Bool {
  var s = new_int_stack();
  if !int_stack_is_empty(&s) { return false; }
  int_stack_push(&mut s, 1);
  int_stack_push(&mut s, 2);
  return !int_stack_is_empty(&s) && int_stack_len(&s) == 2;
}

fn test_factorial_zero() -> Bool {
  return factorial(0) == 1;
}

fn test_factorial_five() -> Bool {
  return factorial(5) == 120;
}

fn test_factorial_seven() -> Bool {
  return factorial(7) == 5040;
}

fn test_safe_divide_normal() -> Bool {
  let r = safe_divide(42, 6);
  if r.is_err() { return false; }
  return r.unwrap() == 7;
}

fn test_safe_divide_by_zero() -> Bool {
  let r = safe_divide(42, 0);
  return r.is_err();
}

fn test_isqrt_perfect_square() -> Bool {
  let r = isqrt(16);
  if r.is_err() { return false; }
  return r.unwrap() == 4;
}

fn test_isqrt_non_square() -> Bool {
  let r = isqrt(10);
  if r.is_err() { return false; }
  var val = r.unwrap();
  return val * val <= 10 && (val + 1) * (val + 1) > 10;
}

fn test_chained_divide() -> Bool {
  let r = chained_divide();
  if r.is_err() { return false; }
  return r.unwrap() == 5;
}

fn test_propagate_zero_div() -> Bool {
  let r = propagate_zero_div();
  return r.is_err();
}

fn test_option_chain_some() -> Bool {
  let r = option_chain(Some(21));
  if r.is_none() { return false; }
  return r.unwrap() == 42;
}

fn test_option_chain_none() -> Bool {
  let r = option_chain(None);
  return r.is_none();
}

fn test_option_chain_zero() -> Bool {
  let r = option_chain(Some(0));
  return r.is_none();
}

fn test_match_enum_with_data() -> Bool {
  var state = AgentState.Running("compile", 50);
  var progress = 0;
  match state {
    AgentState.Running(task, p) => { progress = p; }
    _ => { }
  }
  return progress == 50;
}

fn test_match_enum_multiple_arms() -> Bool {
  var state = AgentState.Done("ok");
  var result = "";
  match state {
    AgentState.Idle => { result = "idle"; }
    AgentState.Running(_, _) => { result = "running"; }
    AgentState.Waiting(_, _) => { result = "waiting"; }
    AgentState.Done(msg) => { result = msg; }
    AgentState.Failed(err) => { result = err; }
  }
  return result == "ok";
}

fn test_while_accumulate() -> Bool {
  var sum = 0;
  var i = 0;
  while i < 10 {
    if i % 2 == 0 {
      sum = sum + i;
    } elif i % 3 == 0 {
      sum = sum + i;
    } else {
      sum = sum + 1;
    }
    i = i + 1;
  }
  return sum == 26;
}

fn test_deeply_nested_if() -> Bool {
  var a = 1;
  var b = 2;
  var c = 3;
  if a < b {
    if b < c {
      if c == 3 {
        if a + b == c {
          return true;
        }
      }
    }
  }
  return false;
}

fn test_vec_push_and_index() -> Bool {
  var v = Vec[Int].new();
  v.push(10);
  v.push(20);
  v.push(30);
  return v[0] == 10 && v[1] == 20 && v[2] == 30 && v.len() == 3;
}

fn test_vec_remove() -> Bool {
  var v = Vec[Int].new();
  v.push(1);
  v.push(2);
  v.push(3);
  v.remove(1);
  return v.len() == 2 && v[0] == 1 && v[1] == 3;
}

// ============================================================================
// Main
// ============================================================================

fn main() -> Int {
  var passed = 0;
  var total = 0;

  total = total + 1;
  if test_agent_new_is_idle() { passed = passed + 1; }

  total = total + 1;
  if test_agent_start_transition() { passed = passed + 1; }

  total = total + 1;
  if test_agent_complete_transition() { passed = passed + 1; }

  total = total + 1;
  if test_agent_fail_transition() { passed = passed + 1; }

  total = total + 1;
  if test_agent_wait_retry_cycle() { passed = passed + 1; }

  total = total + 1;
  if test_agent_wait_zero_retries_rejected() { passed = passed + 1; }

  total = total + 1;
  if test_agent_start_requires_idle() { passed = passed + 1; }

  total = total + 1;
  if test_agent_run_cycle() { passed = passed + 1; }

  total = total + 1;
  if test_stack_push_pop_lifo() { passed = passed + 1; }

  total = total + 1;
  if test_stack_pop_empty() { passed = passed + 1; }

  total = total + 1;
  if test_stack_peek_does_not_remove() { passed = passed + 1; }

  total = total + 1;
  if test_stack_is_empty_and_len() { passed = passed + 1; }

  total = total + 1;
  if test_factorial_zero() { passed = passed + 1; }

  total = total + 1;
  if test_factorial_five() { passed = passed + 1; }

  total = total + 1;
  if test_factorial_seven() { passed = passed + 1; }

  total = total + 1;
  if test_safe_divide_normal() { passed = passed + 1; }

  total = total + 1;
  if test_safe_divide_by_zero() { passed = passed + 1; }

  total = total + 1;
  if test_isqrt_perfect_square() { passed = passed + 1; }

  total = total + 1;
  if test_isqrt_non_square() { passed = passed + 1; }

  total = total + 1;
  if test_chained_divide() { passed = passed + 1; }

  total = total + 1;
  if test_propagate_zero_div() { passed = passed + 1; }

  total = total + 1;
  if test_option_chain_some() { passed = passed + 1; }

  total = total + 1;
  if test_option_chain_none() { passed = passed + 1; }

  total = total + 1;
  if test_option_chain_zero() { passed = passed + 1; }

  total = total + 1;
  if test_match_enum_with_data() { passed = passed + 1; }

  total = total + 1;
  if test_match_enum_multiple_arms() { passed = passed + 1; }

  total = total + 1;
  if test_while_accumulate() { passed = passed + 1; }

  total = total + 1;
  if test_deeply_nested_if() { passed = passed + 1; }

  total = total + 1;
  if test_vec_push_and_index() { passed = passed + 1; }

  total = total + 1;
  if test_vec_remove() { passed = passed + 1; }

  if passed == total { return 0; }
  return 1;
}
