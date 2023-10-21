/// The brainfuck language instructions
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Instruction {
    /// `>` Increment the data pointer by one (to point to the next cell to the right).
    Inc,

    /// `<` Decrement the data pointer by one (to point to the next cell to the left).
    Dec,

    /// `+` Increment the byte at the data pointer by one.
    Succ,

    /// `-` Decrement the byte at the data pointer by one.
    Pred,

    /// ` `Output the byte at the data pointer.
    Out,

    /// `,` Accept one byte of input, storing its value in the byte at the data pointer.
    In,

    /// `[` If the byte at the data pointer is zero, then instead of moving the instruction pointer forward to the next command, jump it forward to the command after the matching ] command.
    Jz {
        /// The value of the data pointer if the jump is taken
        dest: usize,
    },

    /// `]` If the byte at the data pointer is nonzero, then instead of moving the instruction pointer forward to the next command, jump it back to the command after the matching [ command.
    Jnz {
        /// The value of the data pointer if the jump is taken
        dest: usize,
    },
}
