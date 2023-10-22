//! The brainfuck virtual machine

use std::{
    fmt,
    io::{self, Read, Write},
};

use bft_types::{Instruction, Program};

/// The result of executing a single brainfuck command
pub type CommandResult = Result<usize, InterpretError>;

/// The brainfuck virtual machine state
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Machine<'a, Cell: CellKind> {
    /// The program the VM is running
    program: &'a Program,

    /// The memory backing the virtual machine
    tape: Vec<Cell>,

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

/// The bounds required for a type to act as a cell
pub trait CellKind: Default + Clone {
    /// Increment the cell by one, wrapping the result of the computation
    fn wrapping_inc(&mut self);

    /// Decrement the cell by one, wrapping the result of the computation
    fn wrapping_dec(&mut self);

    /// Does this cell contain zero
    fn is_zero(&self) -> bool;

    /// Set the value of the cell
    fn set_value(&mut self, value: u8);

    /// The value of the cell as a slice of bytes
    fn as_bytes(&self) -> Box<[u8]>;
}

/// Implement CellKind for a builtin numeric type
macro_rules! cell_kind_impl {
    ($type:ty) => {
        impl CellKind for $type {
            fn wrapping_inc(&mut self) {
                *self = self.wrapping_add(1);
            }

            fn wrapping_dec(&mut self) {
                *self = self.wrapping_sub(1);
            }

            fn is_zero(&self) -> bool {
                *self == 0
            }

            fn set_value(&mut self, value: u8) {
                *self = value.into();
            }

            fn as_bytes(&self) -> Box<[u8]> {
                Box::new(self.to_be_bytes())
            }
        }
    };
}

/// Implement CellKind for a list of builtin types
macro_rules! cell_kind_impl_all {
    ($($type:ty),+) => {
        $(
            cell_kind_impl!($type);
        )*
    };
}

cell_kind_impl_all!(u8, u16, u32, u64, u128, i16, i32, i64, i128);

#[allow(dead_code)]
impl<'a, Cell: CellKind> Machine<'a, Cell> {
    /// Create a new virtual machine with a growable tape
    ///
    /// `tape_size`: the size of the tape to allocate for the virtual machine
    ///
    /// ```
    /// # use bft_interp::{Machine, TapeKind};
    /// # use bft_types::Program;
    /// let prog = Program::from_file("../programs/example.bf").unwrap();
    /// let vm = Machine::<u8>::new(1000, TapeKind::Growable, &prog);
    /// ```
    pub fn new(tape_size: usize, tape_kind: TapeKind, program: &'a Program) -> Self {
        Self {
            program,
            tape: vec![Cell::default(); tape_size],
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
    /// # use std::io;
    /// let prog = Program::from_file("../programs/example.bf").unwrap();
    /// let mut vm = Machine::<u8>::new(1000, TapeKind::FixedSize, &prog);
    /// vm.run(io::stdin().lock(), io::stdout().lock());
    /// ```
    #[allow(unused_mut)]
    pub fn run(
        &mut self,
        mut input: impl Read,
        mut output: impl Write,
    ) -> Result<(), InterpretError> {
        while let Some(&instr) = self.program.instructions().get(self.ip) {
            self.ip = match instr {
                Instruction::Inc => self.move_head_right()?,
                Instruction::Dec => self.move_head_left()?,
                Instruction::Succ => self.increment_cell()?,
                Instruction::Pred => self.decrement_cell()?,
                Instruction::In => self.read_value(&mut input)?,
                Instruction::Out => self.write_value(&mut output)?,
                Instruction::Jz { dest } => self.jump_if_zero(dest)?,
                Instruction::Jnz { pair_loc } => pair_loc,
            };
        }

        Ok(())
    }

    /// Move the tape head one position to the left
    ///
    /// If the tape head runs off the end TapeRunOffError is returned
    fn move_head_left(&mut self) -> CommandResult {
        match self.dp.checked_sub(1) {
            Some(new_dp) => {
                self.dp = new_dp;
                Ok(self.ip + 1)
            }
            None => Err(InterpretError::TapeRunOffError {
                ip_at_error: self.ip,
            }),
        }
    }

    /// Move the tape head one position to the right
    ///
    /// If the tape head runs off the end TapeRunOffError is returned
    fn move_head_right(&mut self) -> CommandResult {
        self.dp += 1;
        if self.dp >= self.tape.len() {
            if self.tape_can_grow {
                self.tape.reserve(1);
                self.tape.resize(self.tape.capacity(), Cell::default());
            } else {
                // move head left doesn't affect the dp on error
                // we should behave the same
                self.dp -= 1;
                return Err(InterpretError::TapeRunOffError {
                    ip_at_error: self.ip,
                });
            }
        }

        Ok(self.ip + 1)
    }

    /// Increment the value of the cell at the current data pointer
    fn increment_cell(&mut self) -> CommandResult {
        self.tape[self.dp].wrapping_inc();
        Ok(self.ip + 1)
    }

    /// Decrement the value of the cell at the current data pointer
    fn decrement_cell(&mut self) -> CommandResult {
        self.tape[self.dp].wrapping_dec();
        Ok(self.ip + 1)
    }

    /// Read a single u8 from a reader and assign it to the value of the tape
    fn read_value(&mut self, reader: &mut impl Read) -> CommandResult {
        let mut buf = [0u8];
        if let Err(inner) = reader.read_exact(&mut buf) {
            return Err(InterpretError::IoError {
                ip_at_error: self.ip,
                inner,
            });
        };

        self.tape[self.dp].set_value(buf[0]);

        Ok(self.ip + 1)
    }

    /// Write the big-endian byte value of the current cell into the writer
    fn write_value(&mut self, writer: &mut impl Write) -> CommandResult {
        let buf = self.tape[self.dp].as_bytes();

        if let Err(inner) = writer.write_all(&buf) {
            return Err(InterpretError::IoError {
                ip_at_error: self.ip,
                inner,
            });
        };

        Ok(self.ip + 1)
    }

    /// Jump forward if the value of the tape at the data pointer is zerIf the byte at the data pointer is nonzero, then instead of moving the instruction pointer forward to the next command, jump it back to the command after the matching [ command.o
    fn jump_if_zero(&mut self, dest: usize) -> CommandResult {
        if self.tape[self.dp].is_zero() {
            Ok(dest)
        } else {
            Ok(self.ip + 1)
        }
    }
}

/// errors that can occor while interpreting a brainfuck program
#[derive(Debug)]
pub enum InterpretError {
    /// The data pointer exceed the bounds of the tape
    TapeRunOffError {
        /// The instruction which lead to the error
        ip_at_error: usize,
    },

    /// The virtual machine failed to perform an IO operation
    IoError {
        /// The instruction which lead to the error
        ip_at_error: usize,
        /// The inner IO error which caused the failure
        inner: io::Error,
    },
}

impl fmt::Display for InterpretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TapeRunOffError { ip_at_error } => {
                write!(
                    f,
                    "Error: Exceeded the bounds of the tape, IP={ip_at_error}"
                )
            }
            Self::IoError { ip_at_error, inner } => {
                write!(f, "Error: Failed to perform IO ({inner}), IP={ip_at_error}")
            }
        }
    }
}

impl std::error::Error for InterpretError {}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;

    use super::*;

    #[test]
    fn test_move_head_right_grows() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::<u8>::new(1, TapeKind::Growable, &prog);

        for i in 0..100 {
            assert_eq!(machine.dp, i);
            assert!(machine.tape.len() > i);
            let new_ip = machine.move_head_right().unwrap();
            assert_eq!(new_ip, machine.ip + 1);
            machine.ip = new_ip;
            assert_eq!(machine.dp, i + 1);
            assert!(machine.tape.len() > i + 1);
        }
    }

    #[test]
    fn test_move_head_right_run_off() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::<u8>::new(100, TapeKind::FixedSize, &prog);

        for _ in 0..99 {
            let new_ip = machine.move_head_right().unwrap();
            assert_eq!(new_ip, machine.ip + 1);
            machine.ip = new_ip;
        }
        assert!(matches!(
            machine.move_head_right().unwrap_err(),
            InterpretError::TapeRunOffError {
                ip_at_error
            } if ip_at_error == machine.ip
        ));
    }

    #[test]
    fn test_move_head_left() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::<u8>::new(1, TapeKind::Growable, &prog);

        for _ in 0..100 {
            machine.move_head_right().unwrap();
        }
        for i in (0..100).rev() {
            assert_eq!(machine.dp, i + 1);
            let new_ip = machine.move_head_left().unwrap();
            assert_eq!(machine.dp, i);
            assert_eq!(new_ip, machine.ip + 1);
            machine.ip = new_ip;
        }
    }

    #[test]
    fn test_move_head_left_run_off() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::<u8>::new(100, TapeKind::FixedSize, &prog);

        assert!(matches!(
            machine.move_head_left().unwrap_err(),
            InterpretError::TapeRunOffError { ip_at_error: 0 }
        ));
    }

    #[test]
    fn test_increment_cell() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::<u8>::new(100, TapeKind::FixedSize, &prog);

        let new_ip = machine.increment_cell().unwrap();
        assert_eq!(new_ip, machine.ip + 1);
        assert_eq!(machine.tape[0], 1);
        machine.ip = new_ip;

        machine.tape[0].set_value(0xFF);
        let new_ip = machine.increment_cell().unwrap();
        assert_eq!(new_ip, machine.ip + 1);
        assert_eq!(machine.tape[0], 0);
    }

    #[test]
    fn test_decrement_cell() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::<u8>::new(100, TapeKind::FixedSize, &prog);

        let new_ip = machine.decrement_cell().unwrap();
        assert_eq!(new_ip, machine.ip + 1);
        assert_eq!(machine.tape[0], 0xFF);
        machine.ip = new_ip;

        let new_ip = machine.decrement_cell().unwrap();
        assert_eq!(new_ip, machine.ip + 1);
        assert_eq!(machine.tape[0], 0xFE);
    }

    #[test]
    fn test_read_value() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::<u8>::new(100, TapeKind::FixedSize, &prog);

        let data = 0xDEADBEEF_u32.to_be_bytes();
        let mut reader = std::io::Cursor::new(data);
        for d in data {
            let new_ip = machine.read_value(&mut reader).unwrap();
            assert_eq!(machine.tape[0], d);
            assert_eq!(new_ip, machine.ip + 1);
            machine.ip = new_ip;
        }
        machine.read_value(&mut reader).unwrap_err();
    }

    #[test]
    fn test_write_value() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::<u8>::new(100, TapeKind::FixedSize, &prog);

        for i in u8::MIN..=u8::MAX {
            let mut writer = std::io::Cursor::new(vec![0u8]);
            machine.tape[0].set_value(i);
            let new_ip = machine.write_value(&mut writer).unwrap();
            assert_eq!(writer.get_ref(), &[i]);
            assert_eq!(new_ip, machine.ip + 1);
            machine.ip = new_ip;
        }
    }

    #[test]
    fn test_write_value_is_be() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::<u32>::new(100, TapeKind::FixedSize, &prog);

        let val = 0xDEAD_BEEF;
        machine.tape[0] = val;
        let mut writer = std::io::Cursor::new(vec![0u8; 4]);
        for _ in 0..4 {
            let new_ip = machine.write_value(&mut writer).unwrap();
            assert_eq!(new_ip, machine.ip + 1);
            machine.ip = new_ip;
        }

        assert_eq!(&writer.get_ref()[..4], &val.to_be_bytes());
    }

    #[test]
    fn test_is_zero() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::<u8>::new(100, TapeKind::FixedSize, &prog);

        assert!(machine.tape[0].is_zero());
        machine.tape[0].wrapping_inc();
        assert!(!machine.tape[0].is_zero());
    }

    #[test]
    fn test_jump_if_zero() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::<u8>::new(100, TapeKind::FixedSize, &prog);

        assert!(machine.tape[0].is_zero());
        assert_eq!(machine.jump_if_zero(1234).unwrap(), 1234);
        machine.tape[0].wrapping_inc();
        assert!(!machine.tape[0].is_zero());
        assert_eq!(machine.ip, 0);
        assert_eq!(machine.jump_if_zero(1234).unwrap(), 1);
    }

    #[test]
    fn test_run_hello_world() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::<u8>::new(DEFAULT_TAPE_SIZE, TapeKind::FixedSize, &prog);

        let mut output = Vec::new();
        machine.run(io::empty(), &mut output).unwrap();

        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, "hello world");
    }

    #[test]
    fn test_run_rot13() {
        let prog = Program::from_file("../programs/rot13.bf").unwrap();
        let mut machine = Machine::<u8>::new(DEFAULT_TAPE_SIZE, TapeKind::FixedSize, &prog);

        let mut output = Vec::new();
        let input =
            io::Cursor::new(b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789");
        let err = machine.run(input, &mut output).unwrap_err();
        assert!(matches!(err,
            InterpretError::IoError { ip_at_error, inner }
            if ip_at_error == 187 && inner.kind() == ErrorKind::UnexpectedEof
        ));

        let output = String::from_utf8(output).unwrap();
        assert_eq!(
            output,
            "nopqrstuvwxyzabcdefghijklmNOPQRSTUVWXYZABCDEFGHIJKLM0123456789"
        );
    }
}
