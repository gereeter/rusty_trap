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
use breakpoint::{self, Breakpoint};

#[derive(Copy, Clone)]
enum InferiorState {
    Running,
    Stopped,
    SingleStepping
}

pub struct Inferior {
    pub pid: pid_t,
    state: InferiorState,
    current_breakpoint: Option<Breakpoint>
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
            return Ok(Inferior {
                pid: pid,
                state: InferiorState::Running,
                current_breakpoint: None
            }),
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

    pub fn cont<F>(mut self, callback: &mut F) -> i8 where F: FnMut() {
        ptrace_util::cont(self.pid);

        loop {
            match waitpid(self.pid, None) {
                Ok(WaitStatus::Exited(_pid, code)) => return code,
                Ok(WaitStatus::Stopped(_pid, signal::SIGTRAP)) => {
                    let bp = self.current_breakpoint.expect("Hit breakpoint with none set!");
                    match self.state {
                        InferiorState::Running => {
                            callback();
                            breakpoint::step_over(self.pid, bp);
                            self.state = InferiorState::SingleStepping;
                        },
                        InferiorState::SingleStepping => {
                            breakpoint::set(self.pid, bp);
                            ptrace_util::cont(self.pid);
                            self.state = InferiorState::Running;
                        },
                        _ => panic!("Unsupported breakpoint encountered during supported inferior state")
                    }
                }
                Ok(WaitStatus::Stopped(_pid, signal)) => {
                    panic!("Unexpected stop on signal {} in Inferior::continue.  State: {}", signal, self.state as i32)
                },
                Ok(_) => panic!("Unexpected stop in Inferior::continue"),
                Err(_) => panic!("Unhandled error in Inferior::continue")
            };
        }
    }

    pub fn set_breakpoint(&mut self, location: u64) {
        self.current_breakpoint = Some(breakpoint::trap_inferior_set_breakpoint(self.pid, location));
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
