use std::io::{self, Write};

/// Wrap an object implementing Write such that on drop a newline is written if what was written
/// didn't end in a newline
pub struct NewlineWrap<W: Write> {
    /// inner Write instance
    inner: W,
    /// last byte written to the inner instance
    last_written: u8,
}

impl<W: Write> NewlineWrap<W> {
    /// Construct a new NewlineWrap wrapping `writer`
    ///
    /// ```
    /// # use bft_interp::NewlineWrap;
    /// # use std::io::Write;
    /// let mut data_written = Vec::new();
    /// {
    ///     let mut wrapped = NewlineWrap::new(&mut data_written);
    ///     write!(wrapped, "This doesn't end in a newline.").unwrap();
    /// }
    /// assert_eq!(data_written, b"This doesn't end in a newline.\n");
    /// ```
    pub fn new(writer: W) -> Self {
        Self {
            inner: writer,
            last_written: 0,
        }
    }
}

impl<W: Write> Write for NewlineWrap<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let res = self.inner.write(buf)?;
        if let Some(last) = buf.last() {
            self.last_written = *last;
        }

        Ok(res)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<W: Write> Drop for NewlineWrap<W> {
    fn drop(&mut self) {
        if self.last_written != b'\n' {
            self.write_all(b"\n").ok();
            self.flush().ok();
        }
    }
}
