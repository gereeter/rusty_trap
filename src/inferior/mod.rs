use nix::unistd::*;
use nix::unistd::Fork::*;
use libc::pid_t;
use nix::Error;
use nix::errno;
use nix::sys::wait::*;
use std::ffi::CString;
use std::path::Path;
use nix::sys::signal;

use libc::c_void;
use std::ops::{Add, Sub};

use ptrace_util;
use breakpoint::{self, TrapBreakpoint};

#[derive(Copy, Clone)]
pub enum InferiorState {
    Running,
    Stopped,
    SingleStepping
}

pub struct Inferior {
    pub pid: pid_t,
    pub state: InferiorState
}
mod ffi {
    use libc::{c_int, c_long};

    extern {
        pub fn personality(persona: c_long) -> c_int;
    }
}

fn disable_address_space_layout_randomization() {
    unsafe {
        let old = ffi::personality(0xffffffff);
        ffi::personality((old | 0x0040000) as i64);
    }
}

fn exec_inferior(filename: &Path, args: &[&str]) -> ! {
    let c_filename = &CString::new(filename.to_str().unwrap()).unwrap();
    disable_address_space_layout_randomization();
    ptrace_util::trace_me();
    execve(c_filename, &[], &[])
        .ok()
        .expect("Failed execve");
    unreachable!();
}

fn attach_inferior(pid: pid_t) -> Result<Inferior, Error> {
    match waitpid(pid, None) {
        Ok(WaitStatus::Stopped(pid, signal::SIGTRAP)) =>
            return Ok(Inferior {pid: pid, state: InferiorState::Running}),
        Ok(_) => panic!("Unexpected stop in attach_inferior"),
        Err(e) => return Err(e)
    }
}

impl Inferior {
    pub fn exec(filename: &Path, args: &[&str]) -> Result<Inferior, Error> {
        loop {
            match fork() {
                Ok(Child)                      => exec_inferior(filename, args),
                Ok(Parent(pid))                => return attach_inferior(pid),
                Err(Error::Sys(errno::EAGAIN)) => continue,
                Err(e)                         => return Err(e)
            }
        }
    }

    pub fn cont<F>(mut self, callback: &mut F) -> i8
        where F: FnMut(TrapInferior, TrapBreakpoint) {

        ptrace_util::cont(self.pid);
        loop {
            self.state = match waitpid(self.pid, None) {
                Ok(WaitStatus::Exited(_pid, code)) => return code,
                Ok(WaitStatus::Stopped(_pid, signal::SIGTRAP)) =>
                    breakpoint::handle(&mut self, callback),
                Ok(WaitStatus::Stopped(_pid, signal)) => {
                    panic!("Unexpected stop on signal {} in Inferior::continue.  State: {}", signal, self.state as i32)
                },
                Ok(_) => panic!("Unexpected stop in Inferior::continue"),
                Err(_) => panic!("Unhandled error in Inferior::continue")
            };
        }
    }
}

pub type TrapInferior = pid_t;

#[derive(Copy, Clone)]
pub struct InferiorPointer(pub u64);
impl InferiorPointer {
    pub fn as_voidptr(&self) -> * mut c_void {
        let &InferiorPointer(u) = self;
        u as * mut c_void
    }
}
impl Add<i64> for InferiorPointer {
    type Output = InferiorPointer;
    fn add(self, rhs: i64) -> InferiorPointer {
        let InferiorPointer(u) = self;
        if rhs >= 0 {
            InferiorPointer(u + rhs as u64)
        } else {
            InferiorPointer(u - rhs as u64)
        }
    }
}
impl Sub<i64> for InferiorPointer {
    type Output = InferiorPointer;
    fn sub(self, rhs: i64) -> InferiorPointer {
        let InferiorPointer(u) = self;
        if rhs >= 0 {
            InferiorPointer(u - rhs as u64)
        } else {
            InferiorPointer(u + rhs as u64)
        }
    }
}
