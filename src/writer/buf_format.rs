use std::cell::RefCell;
use std::rc::Rc;

use crate::style::{ColorMode, Timestamp};
use crate::writer::{Buffer, Writer};

thread_local! {
    static BUFFER_FORMATTER: RefCell<Option<BufferFormatter>> = const {RefCell::new(None)};
}

pub fn try_with_buf_formatter_slot<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut Option<BufferFormatter>) -> R,
{
    BUFFER_FORMATTER
        .try_with(|tl| {
            let mut slot = tl.try_borrow_mut().ok()?;
            Some(f(&mut slot))
        })
        .ok()
        .flatten()
}

pub struct BufferFormatter {
    buffer: Rc<RefCell<Buffer>>,
    color_mode: ColorMode,
}

impl BufferFormatter {
    pub fn new(writer: &Writer) -> Self {
        Self {
            buffer: Rc::new(RefCell::new(writer.buffer())),
            color_mode: writer.color_mode(),
        }
    }

    pub fn color_mode(&self) -> ColorMode {
        self.color_mode
    }

    pub fn print(&self, writer: &Writer) -> std::io::Result<()> {
        writer.print_out(&self.buffer.borrow())
    }

    pub fn clear(&mut self) {
        self.buffer.borrow_mut().clear();
    }

    pub fn timestamp(&self) -> Timestamp {
        Timestamp::default()
    }
}

impl std::io::prelude::Write for BufferFormatter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.borrow_mut().write_out(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.buffer.borrow_mut().flush()
    }
}
