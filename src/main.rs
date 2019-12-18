use std::env;
use std::ptr;
use std::ffi::{CString, CStr};
use std::process::exit;
use std::result::Result;

extern crate libc;
use libc::timespec;

extern crate nix;
use nix::unistd::*;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitStatus};

#[repr(C)]
struct itimerspec
{
  it_interval: timespec,
  it_value: timespec
}

#[link(name = "rt")]
extern "C"
{
  fn timer_create(clockid: libc::c_int, sevp: *mut libc::sigevent, timerid: *mut libc::c_int) -> libc::c_int;
  fn timer_settime(which: libc::c_int, flags: libc::c_int, new: *mut itimerspec, old: *mut itimerspec) -> libc::c_int;
}

static mut TOU : libc::pid_t = 0;

fn main()
{
  let pid = getpid();
  unsafe
  {
    TOU = -1 * pid.as_raw();
  }
  let args: Vec<String> = env::args().collect();
  let tim = must(sanitize(&args));
  let empty = libc::timespec
  {
    tv_sec: 0,
    tv_nsec: 0
  };
  let cvec: Vec<CString> = args.iter().map(|x| CString::new(x.clone()).unwrap()).collect();
  let cstr: Vec<&CStr> = cvec.iter().map(|x| x.as_c_str()).collect();
  let cargs: &[&CStr] = cstr.as_slice();
  let item = itimerspec
  {
    it_interval: tim,
    it_value: empty
  };

  match fork()
  {
    Ok(ForkResult::Parent { child }) =>
    {
      match waitpid(child, None)
      {
        Ok(WaitStatus::Exited(_,code)) =>{exit(code)}
        _ => {timer_rung()}
      }
    }
    Ok(ForkResult::Child) =>
    {
      let timerid = must(wrap_timer_create());
      must(wrap_timer_settime(timerid, 0, Box::new(item)));

      nmust(execv(&CString::new(args[0].clone()).unwrap(), &cargs[1..]));
    }
    _ => {panic!("What is actually happening?")}
  }
}

fn  wrap_timer_create() -> Result<libc::c_int,&'static str>
{
  let timerid = Box::new(-1);
  let ok;
  let err;
  unsafe
  {
    let t = Box::into_raw(timerid);
    err = timer_create(2, ptr::null_mut(), t);
    ok = *t;
  }
  if err != 0
  {
    return Err("Timer_Create failed badly");
  }
  return Ok(ok);
}

fn wrap_timer_settime(which: libc::c_int, flags: libc::c_int, new: Box<itimerspec>) -> Result<(),&'static str>
{
  let err;
  unsafe
  {
    err = timer_settime(which, flags, Box::into_raw(new), ptr::null_mut());
  }
  if err != 0
  {
    return Err("Failed to set the timer properly");
  }
  return Ok(());
}

fn sanitize(args: &Vec<String>) -> Result<timespec, &'static str>
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
      let u1 = timespec
      {
        tv_sec: x,
        tv_nsec: 0
      };
      return Ok(u1);
    }
  }
}

fn timer_rung() -> ()
{
  unsafe
  {
    let npi = Pid::from_raw(TOU);
    match signal::kill(npi, signal::SIGKILL)
    {
      _ => {panic!("Failed to kill ourselves?");}
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
