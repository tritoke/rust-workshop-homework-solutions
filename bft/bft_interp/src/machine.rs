//! The brainfuck virtual machine

use bft_types::Program;

/// The brainfuck virtual machine state
#[derive(Debug, Clone)]
pub struct Machine {
    /// The memory backing the virtual machine
    tape: Vec<u8>,

    /// Can the tape grow?
    tape_can_grow: bool,
}

/// The default size of the virtual machine's tape
pub const DEFAULT_TAPE_SIZE: usize = 30_000;

impl Default for Machine {
    fn default() -> Self {
        Self::new(DEFAULT_TAPE_SIZE)
    }
}

impl Machine {
    /// Create a new virtual machine with a growable tape
    ///
    /// `tape_size`: the size of the tape to allocate for the virtual machine
    ///
    /// ```
    /// # use bft_interp::Machine;
    /// let vm = Machine::new(1000);
    /// ```
    pub fn new(tape_size: usize) -> Self {
        Self {
            tape: vec![0; tape_size],
            tape_can_grow: true,
        }
    }

    /// Create a new virtual machine with a fixed size tape
    ///
    /// `tape_size`: the size of the tape to allocate for the virtual machine
    ///
    /// ```
    /// # use bft_interp::Machine;
    /// let vm = Machine::new_fixed_size(1000);
    /// ```
    pub fn new_fixed_size(tape_size: usize) -> Self {
        Self {
            tape: vec![0; tape_size],
            tape_can_grow: false,
        }
    }

    /// Create a new virtual machine with a fixed size tape
    ///
    /// `tape_size`: the size of the tape to allocate for the virtual machine
    pub fn run(&self, program: &Program) {
        println!("Running {}", program.filename().display());
        for instr in program.instructions() {
            println!("{instr:?}");
        }
    }
}
