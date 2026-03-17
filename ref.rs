// pub fn log_msg(&self, record_msg: &LogMessage<'_>) {
//     if self.matches(record_msg) {
//         let write_and_flush = |log_formatter: &mut LogFormatter, record_msg: &LogMessage<'_>| {
//             let _ = self
//                 .format
//                 .format_record(log_formatter, record_msg)
//                 .and_then(|_| log_formatter.print(&self.writer));

//             // Clear buffer for next log
//             log_formatter.clear();
//         };

//         //Use thread-local buffer
//         let printed = LOG_FORMATTER
//             .try_with(|tl_buf| {
//                 if let Ok(mut slot) = tl_buf.try_borrow_mut() {
//                     if let Some(ref mut log_formatter) = *slot {
//                         if log_formatter.color_style() != self.writer.color_style() {
//                             *log_formatter = LogFormatter::new(&self.writer);
//                         }
//                         write_and_flush(log_formatter, record_msg);
//                     } else {
//                         let mut log_formatter = LogFormatter::new(&self.writer);
//                         write_and_flush(&mut log_formatter, record_msg);
//                         *slot = Some(log_formatter);
//                     }
//                 } else {
//                     write_and_flush(&mut LogFormatter::new(&self.writer), record_msg);
//                 }
//             })
//             .is_ok();

//         // Fallback if thread-local unavailable (thread shutting down)
//         if !printed {
//             write_and_flush(&mut LogFormatter::new(&self.writer), record_msg);
//         }
//     }
// }

// pub fn flush(&self) {
//     // Flush all thread-local formatters
//     let _ = LOG_FORMATTER.try_with(|tl_buf| {
//         if let Ok(mut slot) = tl_buf.try_borrow_mut() {
//             if let Some(ref mut log_formatter) = *slot {
//                 // print buffer
//                 let _ = log_formatter.print(&self.writer);
//                 // reset buffer
//                 log_formatter.clear();
//             }
//         }
//     });

//     // Flush the underlying writer's buffer
//     let _ = self.writer.flush();
// }
