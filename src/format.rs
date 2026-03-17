use std::cell::RefCell;
use std::fmt::Display;
use std::io::{self, Write};
use std::rc::Rc;

use crate::logger::log_config::LogLevel;
use crate::logger::log_message::LogMessage;
use crate::style::{ColorStyle, Timestamp, TimestampPrecision};
use crate::writer::{Buffer, Writer};

thread_local! {
    static LOG_FORMATTER: RefCell<Option<LogFormatter>> = const {RefCell::new(None)};
}

pub fn get_log_formatter<F, R>(writer: &Writer, f: F) -> R
where
    F: FnOnce(&mut LogFormatter) -> R,
{
    let mut result: Option<R> = None;

    // Access thread-local LOG_FORMATTER
    let _ = LOG_FORMATTER.try_with(|tl_buf| {
        if let Ok(mut slot) = tl_buf.try_borrow_mut() {
            match slot.as_mut() {
                Some(log_formatter) => {
                    // Recreate if color_style changed
                    if log_formatter.color_style() != writer.color_style() {
                        *log_formatter = LogFormatter::new(writer);
                    }
                    result = Some(f(log_formatter));
                }
                None => {
                    let mut log_formatter = LogFormatter::new(writer);
                    result = Some(f(&mut log_formatter));
                    *slot = Some(log_formatter);
                }
            }
        } else {
            // Fallback if already borrowed
            let mut log_formatter = LogFormatter::new(writer);
            result = Some(f(&mut log_formatter));
        }
    });

    // Fallback if thread-local unavailable (thread shutting down)
    if result.is_none() {
        let mut log_formatter = LogFormatter::new(writer);
        result = Some(f(&mut log_formatter));
    }

    // Safe unwrap because we always set result
    result.unwrap()
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
