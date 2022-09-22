#![no_main]

use std::time::{Instant};

extern crate libc;
use libc::{rlimit, rlim_t, __rlimit_resource_t};
use libc::setrlimit;
use libc::execvp;
use libc::c_char;

macro_rules! clean_panic
{
  ($str:literal) => {
    println!("Error: {}", $str);
    std::process::exit(1)
  }
}

#[no_mangle]
pub extern fn main(argc: i32, argv: *const *const c_char) -> i32
{
  let start = Instant::now();
  let time = sanitize_input(argc, argv);

  let mut rlp = rlimit
  {
    rlim_cur: time,
    rlim_max: time+1
  };
  let rlpb: *const rlimit = &rlp;

  let s = start.elapsed().as_secs();
  rlp.rlim_cur = rlp.rlim_cur + s;
  rlp.rlim_max = rlp.rlim_max + s;
    
  //unsafe: pushing through argv to execvp
  let function = unsafe{*argv.offset(2)};
  let execvp_args = unsafe{argv.offset(2)};  
  wrap_setrlimit(libc::RLIMIT_CPU, rlpb);
  unsafe{execvp(function, execvp_args)}
}
/*
#[inline]
fn wrap_execvp(function: *const c_char, args: *const *const c_char) -> i32
{
  //unsafe: call to execvp
  unsafe
  {
    execvp(function, args);
  }
}
*/
fn wrap_setrlimit(resource: __rlimit_resource_t, rlp: *const rlimit)
{
  //unsafe: call to setrlimit
  let setrlimit_status;  
  unsafe
  {
    setrlimit_status = setrlimit(resource, rlp);
  };

  if setrlimit_status > 0 {
    clean_panic!("failed to set rlimit");
  }
}

fn sanitize_input(argc: i32, argv: *const *const c_char) -> rlim_t
{
  if 2 > argc
  {
    clean_panic!("you must provide two arguments: runout [seconds] [COMMAND]");
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
      Some(a) => { secs *= 10; secs += a as rlim_t; }
      None => { clean_panic!("the first argument must be a valid time_t number"); }
    };
    i+=1;
    //unsafe: get the ith char
    curr = unsafe{*num.offset(i)};
  }
  return secs;
}

