use ptrace_util::*;
use inferior::*;

pub type TrapBreakpoint = i32;

#[derive(Copy, Clone)]
pub struct Breakpoint {
    shift : u64,
    target_address  : InferiorPointer,
    aligned_address : InferiorPointer,
    original_breakpoint_word : i64
}

pub fn step_over(inferior: TrapInferior, bp: Breakpoint) -> () {
    poke_text(inferior, bp.aligned_address, bp.original_breakpoint_word);
    set_instruction_pointer(inferior, bp.target_address);
    single_step(inferior);
}

pub fn set(inferior: TrapInferior, bp: Breakpoint) -> () {
    let mut modified = bp.original_breakpoint_word;
    modified &= !0xFFi64 << bp.shift;
    modified |= 0xCCi64 << bp.shift;
    poke_text(inferior, bp.aligned_address, modified);
}

pub fn trap_inferior_set_breakpoint(inferior: TrapInferior, location: u64) -> Breakpoint {
    let aligned_address = location & !0x7u64;
    let bp = Breakpoint {
        shift : (location - aligned_address) * 8,
        aligned_address: InferiorPointer(aligned_address),
        target_address: InferiorPointer(location),
        original_breakpoint_word: peek_text(inferior, InferiorPointer(aligned_address))
    };

    set(inferior, bp);

    bp
}
