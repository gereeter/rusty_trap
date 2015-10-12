extern crate nix;
extern crate libc;

mod ptrace_util;

mod inferior;
pub use inferior::Inferior;
use inferior::InferiorPointer;

mod breakpoint;

pub use self::breakpoint::trap_inferior_set_breakpoint;
use breakpoint::{handle, TrapBreakpoint};

#[derive(Copy, Clone)]
struct Breakpoint {
    shift : u64,
    target_address  : InferiorPointer,
    aligned_address : InferiorPointer,
    original_breakpoint_word : i64
}

static mut global_breakpoint : Breakpoint = Breakpoint {
    shift: 0,
    target_address: InferiorPointer(0),
    aligned_address: InferiorPointer(0),
    original_breakpoint_word: 0
};
