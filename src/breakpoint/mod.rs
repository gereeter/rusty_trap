use ptrace_util;
use inferior::{TrapInferior, InferiorPointer};

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
            original_breakpoint_word: ptrace_util::peek_text(inferior, InferiorPointer(aligned_address))
        }
    }

    pub fn step_over(self, inferior: TrapInferior) {
        self.unset(inferior);
        ptrace_util::set_instruction_pointer(inferior, self.target_address);
        ptrace_util::single_step(inferior);
    }

    pub fn unset(self, inferior: TrapInferior) {
        ptrace_util::poke_text(inferior, self.aligned_address, self.original_breakpoint_word);
    }

    pub fn set(self, inferior: TrapInferior) {
        let mut modified = self.original_breakpoint_word;
        modified &= !0xFFi64 << self.shift;
        modified |= 0xCCi64 << self.shift;
        ptrace_util::poke_text(inferior, self.aligned_address, modified);
    }
}
