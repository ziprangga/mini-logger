use std::cell::RefCell;
use std::rc::Rc;

use crate::style::{ColorStyle, Timestamp};
use crate::writer::{Buffer, Writer};

thread_local! {
    static LOG_FORMATTER: RefCell<Option<LogFormatter>> = const {RefCell::new(None)};
}

pub fn try_with_log_formatter_slot<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut Option<LogFormatter>) -> R,
{
    LOG_FORMATTER
        .try_with(|tl| {
            let mut slot = tl.try_borrow_mut().ok()?;
            Some(f(&mut slot))
        })
        .ok()
        .flatten()
}

pub struct LogFormatter {
    buffer: Rc<RefCell<Buffer>>,
    color_style: ColorStyle,
}

impl LogFormatter {
    pub fn new(writer: &Writer) -> Self {
        Self {
            buffer: Rc::new(RefCell::new(writer.buffer())),
            color_style: writer.color_style(),
        }
    }

    pub fn color_style(&self) -> ColorStyle {
        self.color_style
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

impl std::io::prelude::Write for LogFormatter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.borrow_mut().write_out(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.buffer.borrow_mut().flush()
    }
}

pub fn log_msg(&self, record_msg: &LogMessage<'_>) {
    if !self.matches(record_msg) {
        return;
    }

    let write_and_flush = |log_formatter: &mut LogFormatter| {
        let _ = self
            .format
            .format_record(log_formatter, record_msg)
            .and_then(|_| log_formatter.print(&self.writer));
        log_formatter.clear();
    };

    let printed = try_with_log_formatter_slot(|log_formatter_slot| match log_formatter_slot {
        Some(log_formatter) => {
            if log_formatter.color_style() != self.writer.color_style() {
                *log_formatter = LogFormatter::new(&self.writer);
            }
            write_and_flush(log_formatter);
        }
        slot @ None => {
            let mut log_formatter = LogFormatter::new(&self.writer);
            write_and_flush(&mut log_formatter);
            *slot = Some(log_formatter);
        }
        None => {
            let mut log_formatter = LogFormatter::new(&self.writer);
            write_and_flush(&mut log_formatter);
            *log_formatter_slot = Some(log_formatter);
        }
    })
    .is_some();

    // Fallback if thread-local unavailable (thread shutting down)
    if !printed {
        let mut log_formatter = LogFormatter::new(&self.writer);
        write_and_flush(&mut log_formatter);
    }
}

pub fn flush(&self) {
    let _ = try_with_log_formatter_slot(|log_formatter_slot| {
        if let Some(log_formatter) = log_formatter_slot {
            let _ = log_formatter.print(&self.writer);
            log_formatter.clear();
        }
    });

    let _ = self.writer.flush();
}
