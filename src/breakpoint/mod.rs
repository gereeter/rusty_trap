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

impl Breakpoint {
    pub fn new(location: u64, inferior: TrapInferior) -> Breakpoint {
        let aligned_address = location & !0x7u64;
        Breakpoint {
            shift : (location - aligned_address) * 8,
            aligned_address: InferiorPointer(aligned_address),
            target_address: InferiorPointer(location),
            original_breakpoint_word: peek_text(inferior, InferiorPointer(aligned_address))
        }
    }

    pub fn step_over(self, inferior: TrapInferior) {
        poke_text(inferior, self.aligned_address, self.original_breakpoint_word);
        set_instruction_pointer(inferior, self.target_address);
        single_step(inferior);
    }

    pub fn set(self, inferior: TrapInferior) {
        let mut modified = self.original_breakpoint_word;
        modified &= !0xFFi64 << self.shift;
        modified |= 0xCCi64 << self.shift;
        poke_text(inferior, self.aligned_address, modified);
    }
}
