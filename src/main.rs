use libc::*;
use std::ptr::null_mut;
use nix::sys::signal::*;
use std::mem;
use std::ptr;
use std::ptr::from_mut;
use std::env;
use nix::fcntl::Flock;
use nix::fcntl::FlockArg;
use std::fs::File;

extern "C" fn dummy(_: c_int) -> () { () }

fn main() -> Result<(),std::io::Error> {
   #[cfg(target_os = "linux")]
   {
      let mut cmdline_arg = env::args().fuse();
      cmdline_arg.next().expect("Something is wrong here...");
      let flock_target = match cmdline_arg.next() {
         Some(arg) => arg,
         None => {
            println!("Please, supply a file path to attempt flock() for!");
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "No input"));
         }
      };
      let f_handle = File::open(flock_target)?;
      let event_value: sigval = sigval { sival_ptr: ptr::null_mut() };
      let mut event: sigevent = unsafe { mem::zeroed() };
      event.sigev_value = event_value;
      event.sigev_signo = SIGRTMIN()+3;
      event.sigev_notify = SIGEV_THREAD_ID;
      event.sigev_notify_thread_id = unsafe { gettid() };
      let mut t: timer_t = null_mut();
      let n: timespec = timespec {tv_sec: 3, tv_nsec: 0};
      let zero: timespec = timespec { tv_sec: 0, tv_nsec: 0};
      let s: itimerspec = itimerspec {it_interval: zero, it_value: n};
      let handler = SigHandler::Handler(dummy);
      let act = SigAction::new(handler, SaFlags::empty(), SigSet::empty());
       unsafe {
       	timer_create(CLOCK_MONOTONIC, from_mut(&mut event), &mut t);
       	timer_settime(t, 0, &s, null_mut());
       	let _ = nix::sys::signal::rt_sigaction(SignalValue::Realtime(3), &act)?;
       }
      let _lock_guard = match Flock::lock(f_handle, FlockArg::LockExclusive) {
         Ok(_) => {
            println!("Success");
            return Ok(());
         }
         Err((_, error)) => {
            println!("flock has failed: {}", error);
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "flock has failed"));
         }
      };
       // thread::sleep(Duration::from_secs(86400)); // -- restarts if interrupted by a signal
   }
   #[cfg(not(target_os = "linux"))]
   {
      panic!("This is a Linux-only project");
   }
}
