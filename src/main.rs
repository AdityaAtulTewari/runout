use std::env;
use std::ffi::{CString, CStr};
use std::time::{Instant};
use std::result::Result;

extern crate libc;
use libc::{rlimit, rlim_t, c_int};

extern crate nix;
use nix::unistd::*;

static RLIMIT_CPU: c_int = 0;

#[link(name = "c")]
extern "C"
{
  fn setrlimit(resource: c_int, rlp: *mut rlimit) -> c_int;
}

fn main()
{
  let start = Instant::now();
  let args: Vec<String> = env::args().collect();
  let cvec: Vec<CString> = args.iter().map(|x| CString::new(x.clone()).unwrap()).collect();
  let cstr: Vec<&CStr> = cvec.iter().map(|x| x.as_c_str()).collect();
  let cargs: &[&CStr] = cstr.as_slice();
  let toexec = &CString::new(args[2].clone()).unwrap();
  let slice = &cargs[2..];
  let time = must(sanitize(&args));
  let s = start.elapsed().as_secs();
  let rlp = rlimit {rlim_cur: time +s, rlim_max: time+s+1};
  let rlpb = Box::new(rlp);
  must(wrap_setrlimit(RLIMIT_CPU, rlpb));
  println!("{:?}", toexec);
  println!("{:?}", slice);
  nmust(execvp(toexec, slice));
}

fn wrap_setrlimit(resource: c_int, rlp: Box<rlimit>) -> Result<(), &'static str>
{
  let err;
  unsafe
  {
    let urlp = Box::into_raw(rlp);
    err = setrlimit(resource, urlp);
  }
  if err != 0
  {
    return Err("Failed to set limits");
  }
  return Ok(());
}

fn sanitize(args: &Vec<String>) -> Result<rlim_t, &'static str>
{
  if 3 > args.len()
  {
    return Err("You must provide three arguments, runout [seconds] [COMMAND]");
  }

  let secs = args[1].parse();

  match secs
  {
    Err(_) => {return Err("The second argument must be a valid time_t number.")}
    Ok(x) =>
    {
      return Ok(x);
    }
  }
}

fn nmust<T>(result: nix::Result<T>) -> T
{
  match result
  {
    Ok(x) => {return x}
    Err(x) => {panic!("{:?}", x)}
  }
}

fn must<T, E: std::fmt::Display>(result: Result<T,E>) -> T
{
  match result
  {
    Ok(x) => {return x}
    Err(x) => {panic!("{}", x)}
  }
}

#[cfg(test)]
mod tests
{
  use super::*;

  /*
   * We would like to test the sanitization on a few particular inputs.
   */
  #[test]
  fn test_sanitize()
  {
    assert_eq!(sanitize(&vec!["-w".to_string(), "60".to_string()]), Err("You must provide three arguments, runout [seconds] [COMMAND]"));
    assert_eq!(sanitize(&vec!["-w".to_string(), "a".to_string(), "asdfa".to_string()]), Err("The second argument must be a valid time_t number."));
  }
}
