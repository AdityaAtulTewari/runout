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
  let time = sanitize_input(argc, argv).unwrap();

  let mut rlp = rlimit
  {
    rlim_cur: time,
    rlim_max: time+1
  };
  let rlpb: *const rlimit = &rlp;

  let s = start.elapsed().as_secs();
  rlp.rlim_cur = rlp.rlim_cur + s;
  rlp.rlim_max = rlp.rlim_max + s;

  let function = unsafe{*argv.offset(2)};
  let execvp_args = unsafe{argv.offset(2)};  
  //unsafe: pushing through argv to execvp

  wrap_setrlimit(libc::RLIMIT_CPU, rlpb).unwrap();
  return wrap_execvp(function, execvp_args).unwrap();
}

#[inline]
fn wrap_execvp(function: *const c_char, args: *const *const c_char) -> Result<(), &'static str>
{
  //unsafe: call to execvp
  let execvp_status;
  unsafe
  {
    execvp_status = execvp(function, args);
  };

  match execvp_status
  {
    0 => Ok(()),
    _ => Err("Failed to set limits")
  }
}

fn wrap_setrlimit(resource: __rlimit_resource_t, rlp: *const rlimit) -> Result<(), &'static str>
{
  //unsafe: call to setrlimit
  let setrlimit_status;  
  unsafe
  {
    setrlimit_status = setrlimit(resource, rlp);
  };

  match setrlimit_status
  {
    0 => Ok(()),
    _ => Err("Failed to set limits")
  }
}

fn sanitize_input(argc: i32, argv: *const *const c_char) -> Result<rlim_t, &'static str>
{
  if 2 != argc
  {
    return Err("You must provide two arguments: runout [seconds] [COMMAND]");
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
      None => { return Err("The second argument must be a valid time_t number."); }
    };
    i+=1;
    //unsafe: get the ith char
    curr = unsafe{*num.offset(i)};
  }
  return Ok(secs);
}

