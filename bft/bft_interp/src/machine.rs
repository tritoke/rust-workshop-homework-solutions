//! The brainfuck virtual machine

use bft_types::Program;

/// The brainfuck virtual machine state
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Machine<'a> {
    /// The program the VM is running
    program: &'a Program,

    /// The memory backing the virtual machine
    tape: Vec<u8>,

    /// Can the tape grow?
    tape_can_grow: bool,

    /// The current location of the head of the tape
    dp: usize,

    /// The current location of the head of the tape
    ip: usize,
}

/// The default size of the virtual machine's tape
pub const DEFAULT_TAPE_SIZE: usize = 30_000;

/// The kinds of tape the virtual machine supports
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapeKind {
    /// a growable tape
    Growable,
    /// a fixed-size tape
    FixedSize,
}

#[allow(dead_code)]
impl<'a> Machine<'a> {
    /// Create a new virtual machine with a growable tape
    ///
    /// `tape_size`: the size of the tape to allocate for the virtual machine
    ///
    /// ```
    /// # use bft_interp::{Machine, TapeKind};
    /// # use bft_types::Program;
    /// let prog = Program::from_file("../programs/example.bf").unwrap();
    /// let vm = Machine::new(1000, TapeKind::Growable, &prog);
    /// ```
    pub fn new(tape_size: usize, tape_kind: TapeKind, program: &'a Program) -> Self {
        Self {
            program,
            tape: vec![0; tape_size],
            tape_can_grow: tape_kind == TapeKind::Growable,
            dp: 0,
            ip: 0,
        }
    }

    /// Create a new virtual machine with a fixed size tape
    ///
    /// `tape_size`: the size of the tape to allocate for the virtual machine
    ///
    /// ```
    /// # use bft_interp::{Machine, TapeKind};
    /// # use bft_types::Program;
    /// let prog = Program::from_file("../programs/example.bf").unwrap();
    /// let mut vm = Machine::new(1000, TapeKind::FixedSize, &prog);
    /// vm.run();
    /// ```
    #[allow(unused_mut)]
    pub fn run(&mut self) {
        println!("Running {}", self.program.filename().display());
        for instr in self.program.instructions() {
            println!("{instr:?}");
        }
    }

    /// Move the tape head one position to the left
    ///
    /// If the tape head runs off the end TapeRunOffError is returned
    fn move_head_left(&mut self) -> Result<(), InterpretError> {
        match self.dp.checked_sub(1) {
            Some(new_dp) => {
                self.dp = new_dp;
                Ok(())
            }
            None => Err(InterpretError::TapeRunOffError {
                ip_at_error: self.ip,
            }),
        }
    }

    /// Move the tape head one position to the right
    ///
    /// If the tape head runs off the end TapeRunOffError is returned
    fn move_head_right(&mut self) -> Result<(), InterpretError> {
        self.dp += 1;
        if self.dp >= self.tape.len() {
            if self.tape_can_grow {
                self.tape.reserve(1);
                self.tape.resize(self.tape.capacity(), 0);
            } else {
                return Err(InterpretError::TapeRunOffError {
                    ip_at_error: self.ip,
                });
            }
        }

        Ok(())
    }
}

/// errors that can occor while interpreting a brainfuck program
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpretError {
    /// The data pointer exceed the bounds of the tape
    TapeRunOffError {
        /// The instruction which lead to the error
        ip_at_error: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_head_right_grows() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::new(1, TapeKind::Growable, &prog);

        for i in 0..100 {
            assert_eq!(machine.dp, i);
            assert!(machine.tape.len() > i);
            machine.move_head_right().unwrap();
            assert_eq!(machine.dp, i + 1);
            assert!(machine.tape.len() > i + 1);
        }
    }

    #[test]
    fn test_move_head_right_run_off() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::new(100, TapeKind::FixedSize, &prog);

        for _ in 0..99 {
            machine.move_head_right().unwrap();
        }
        assert_eq!(
            machine.move_head_right().unwrap_err(),
            InterpretError::TapeRunOffError { ip_at_error: 0 }
        );
    }

    #[test]
    fn test_move_head_left() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::new(1, TapeKind::Growable, &prog);

        for _ in 0..100 {
            machine.move_head_right().unwrap();
        }
        for i in (0..100).rev() {
            assert_eq!(machine.dp, i + 1);
            machine.move_head_left().unwrap();
            assert_eq!(machine.dp, i);
        }
    }

    #[test]
    fn test_move_head_left_run_off() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::new(100, TapeKind::FixedSize, &prog);

        for _ in 0..99 {
            machine.move_head_right().unwrap();
        }
        assert_eq!(
            machine.move_head_right().unwrap_err(),
            InterpretError::TapeRunOffError { ip_at_error: 0 }
        );
    }
}
