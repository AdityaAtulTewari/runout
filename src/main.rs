#![no_main]

use std::time::{Instant};
use std::result::Result;

extern crate libc;
use libc::{rlimit, rlim_t, __rlimit_resource_t};
use libc::setrlimit;
use libc::execvp;
use libc::c_char;

#[no_mangle]
pub extern fn main(argc: i32, argv: *const *const c_char) -> i32
{
  let start = Instant::now();
  let time = must(sanitize(argc, argv));
  let mut rlp = rlimit {rlim_cur: time, rlim_max: time+1};
  let rlpb: *const rlimit = &rlp;
  let s = start.elapsed().as_secs();
  rlp.rlim_cur = rlp.rlim_cur + s;
  rlp.rlim_max = rlp.rlim_max + s;
  must(wrap_setrlimit(libc::RLIMIT_CPU, rlpb));
  //unsafe: pushing through argv to execvp
  must(wrap_execvp(unsafe{*argv.offset(2)}, unsafe{argv.offset(2)}));
  return -1;
}

#[inline]
fn wrap_execvp(function: *const c_char, args: *const *const c_char) -> Result<(), &'static str>
{
  //unsafe: call to execvp
  unsafe
  {
    execvp(function, args);
  }
  return Err("Failed to exec");
}

fn wrap_setrlimit(resource: __rlimit_resource_t, rlp: *const rlimit) -> Result<(), &'static str>
{
  let err;
  //unsafe: call to setrlimit
  unsafe
  {
    err = setrlimit(resource, rlp);
  }
  if err != 0
  {
    return Err("Failed to set limits");
  }
  return Ok(());
}

fn sanitize(argc: i32, argv: *const *const c_char) -> Result<rlim_t, &'static str>
{
  if 2 > argc
  {
    return Err("You must provide three arguments, runout [seconds] [COMMAND]");
  }
  let num: *const c_char;
  //unsafe: parsing argument 1 in a clike fashion.
  unsafe
  {
    num = *argv.offset(1);
  }
  //unsafe: get zeroth char
  let mut curr: c_char = unsafe{*num};
  let mut i: isize = 0;
  let mut secs: rlim_t = 0;

  while curr != 0
  {
    match (curr as u8 as char).to_digit(10)
    {
      Some(a) => {secs *= 10; secs += a as rlim_t;}
      None => { return Err("The second argument must be a valid time_t number.");}
    }
    i+=1;
    //unsafe: get the ith char
    curr = unsafe{*num.offset(i)};
  }
  return Ok(secs);
}

fn must<T, E: std::fmt::Display>(result: Result<T,E>) -> T
{
  match result
  {
    Ok(x) => {return x}
    Err(x) => {panic!("{}", x)}
  }
}
