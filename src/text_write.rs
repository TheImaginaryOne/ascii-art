use std::io::Write;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TextWriteError {
    // TODO add an ImageResult.
    #[error("std::io error")]
    StdIo(#[from] std::io::Error)
}

pub trait TextWrite<T> {
    fn flush(&mut self) -> Result<(), T>;

    fn write_char(&mut self, c: char) -> Result<(), T>;

    fn write_newline(&mut self) -> Result<(), T>;
}

pub struct StdTextWriter<T: Write> {
    writer: T,
}
impl<T: Write> StdTextWriter<T> {
    pub fn new(writer: T) -> Self {
        Self {
            writer
        }
    }
}
impl<T: Write> TextWrite<TextWriteError> for StdTextWriter<T> {
    fn flush(&mut self) -> Result<(), TextWriteError> {
        self.writer.flush()?;
        Ok(())
    }

    fn write_char(&mut self, c: char) -> Result<(), TextWriteError> {
        let mut b = [0; 4];
        let slice = c.encode_utf8(&mut b);
        self.writer.write(&slice.as_bytes())?;
        Ok(())
    }

    fn write_newline(&mut self) -> Result<(), TextWriteError> {
        self.writer.write(b"\n")?;
        Ok(())
    }
}
