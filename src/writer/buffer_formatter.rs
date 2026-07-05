use std::cell::RefCell;
use std::rc::Rc;

use crate::style::{ColorMode, Timestamp};
use crate::writer::{Buffer, Writer};

thread_local! {
    static BUFFER_FORMATTER: RefCell<Option<BufferFormatter>> = const {RefCell::new(None)};
}

/// Executes a closure with access to a thread-local [`BufferFormatter`] slot.
///
/// This slot is used to reuse a formatter per thread to avoid allocating a
/// new buffer for every log record.
///
/// If the thread-local storage is unavailable (e.g. during thread shutdown),
/// returns `None`.
///
/// The slot may contain an existing formatter, or be empty if this is the
/// first log call on the thread.
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

/// Formatter backed by an in-memory buffer.
///
/// Formatting writes into the buffer through the standard [`std::io::Write`]
/// interface. The completed buffer can later be written by a [`Writer`].
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

    /// Returns a timestamp representing the current time.
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
