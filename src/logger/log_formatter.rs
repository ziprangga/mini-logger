use std::cell::RefCell;
use std::rc::Rc;
use std::time::SystemTime;

use crate::style::{ColorStyle, Timestamp, TimestampPrecision};
use crate::writer::{Buffer, BufferWriter};

pub struct LogFormatter {
    buffer: Rc<RefCell<Buffer>>,
    color_style: ColorStyle,
}

impl std::io::prelude::Write for LogFormatter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.buffer.borrow_mut().flush()
    }
}

impl LogFormatter {
    pub fn new(writer: &BufferWriter) -> Self {
        Self {
            buffer: Rc::new(RefCell::new(writer.buffer())),
            color_style: writer.color_style(),
        }
    }

    pub fn color_style(&self) -> ColorStyle {
        self.color_style
    }

    pub fn write(&self, writer: &BufferWriter) -> std::io::Result<()> {
        writer.write(&self.buffer.borrow())
    }

    pub fn clear(&mut self) {
        self.buffer.borrow_mut().clear();
    }

    pub fn timestamp(&self, precision: TimestampPrecision) -> Timestamp {
        Timestamp {
            time: SystemTime::now(),
            precision,
        }
    }
}
