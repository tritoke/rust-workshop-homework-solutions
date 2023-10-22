//! The brainfuck virtual machine

use std::io::{self, Read, Write};

use bft_types::Program;

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
    /// let prog = Program::from_file("../programs/example.bf").unwrap();
    /// let mut vm = Machine::<u8>::new(1000, TapeKind::FixedSize, &prog);
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

        Ok(())
    }

    /// Increment the value of the cell at the current data pointer
    fn increment_cell(&mut self) {
        self.tape[self.dp].wrapping_inc()
    }

    /// Decrement the value of the cell at the current data pointer
    fn decrement_cell(&mut self) {
        self.tape[self.dp].wrapping_dec()
    }

    /// Read a single u8 from a reader and assign it to the value of the tape
    fn read_value(&mut self, reader: &mut impl Read) -> Result<(), InterpretError> {
        let mut buf = [0u8];
        if let Err(inner) = reader.read_exact(&mut buf) {
            return Err(InterpretError::IoError {
                ip_at_error: self.ip,
                inner,
            });
        };

        self.tape[self.dp].set_value(buf[0]);

        Ok(())
    }

    /// Write the big-endian byte value of the current cell into the writer
    fn write_value(&mut self, writer: &mut impl Write) -> Result<(), InterpretError> {
        let buf = self.tape[self.dp].as_bytes();

        if let Err(inner) = writer.write_all(&buf) {
            return Err(InterpretError::IoError {
                ip_at_error: self.ip,
                inner,
            });
        };

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_head_right_grows() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::<u8>::new(1, TapeKind::Growable, &prog);

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
        let mut machine = Machine::<u8>::new(100, TapeKind::FixedSize, &prog);

        for _ in 0..99 {
            machine.move_head_right().unwrap();
        }
        assert!(matches!(
            machine.move_head_right().unwrap_err(),
            InterpretError::TapeRunOffError { ip_at_error: 0 }
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
            machine.move_head_left().unwrap();
            assert_eq!(machine.dp, i);
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

        machine.increment_cell();
        assert_eq!(machine.tape[0], 1);

        machine.tape[0].set_value(0xFF);
        machine.increment_cell();
        assert_eq!(machine.tape[0], 0);
    }

    #[test]
    fn test_decrement_cell() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::<u8>::new(100, TapeKind::FixedSize, &prog);

        machine.decrement_cell();
        assert_eq!(machine.tape[0], 0xFF);
        machine.decrement_cell();
        assert_eq!(machine.tape[0], 0xFE);
    }

    #[test]
    fn test_read_value() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::<u8>::new(100, TapeKind::FixedSize, &prog);

        let data = 0xDEADBEEF_u32.to_be_bytes();
        let mut reader = std::io::Cursor::new(data);
        for d in data {
            machine.read_value(&mut reader).unwrap();
            assert_eq!(machine.tape[0], d);
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
            machine.write_value(&mut writer).unwrap();
            assert_eq!(writer.get_ref(), &[i]);
        }
    }

    #[test]
    fn test_write_value_is_be() {
        let prog = Program::from_file("../programs/example.bf").unwrap();
        let mut machine = Machine::<u32>::new(100, TapeKind::FixedSize, &prog);

        let val = 0xDEAD_BEEF;
        machine.tape[0] = val;
        let mut writer = std::io::Cursor::new(vec![0u8; 4]);
        machine.write_value(&mut writer).unwrap();
        machine.write_value(&mut writer).unwrap();
        machine.write_value(&mut writer).unwrap();
        machine.write_value(&mut writer).unwrap();

        assert_eq!(&writer.get_ref()[..4], &val.to_be_bytes());
    }
}
