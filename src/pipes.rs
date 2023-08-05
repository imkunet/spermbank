use std::io::{self, Write};

use humansize::{FormatSize, FormatSizeOptions};
use tracing::info;

pub struct MeteredPipe<'a, T: Write> {
    pub label: String,
    pub destination: &'a mut T,
    pub size: u64,
}

impl<'a, T: Write> MeteredPipe<'a, T> {
    pub fn new(label: &str, write: &'a mut T) -> MeteredPipe<'a, T> {
        MeteredPipe::<'a, T> {
            label: label.to_string(),
            destination: write,
            size: 0,
        }
    }
}

impl<'a, T: Write> Write for MeteredPipe<'a, T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.destination.write(buf)?;
        self.size += written as u64;

        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.destination.flush()?;
        info!(
            "{} flushed {} ({})",
            self.label,
            self.size
                .format_size(FormatSizeOptions::from(humansize::DECIMAL).space_after_value(false)),
            self.size
        );
        self.size = 0;
        Ok(())
    }
}
