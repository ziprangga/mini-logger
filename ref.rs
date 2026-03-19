// use std::io;

// // your trait
// pub trait FormatRecord {
//     fn format_record(
//         &self,
//         buf: &mut BufferFormatter,
//         msg: &LogMessage<'_>,
//     ) -> io::Result<()>;
// }

// // blanket impl for closures
// impl<F> FormatRecord for F
// where
//     F: Fn(&mut BufferFormatter, &LogMessage<'_>) -> io::Result<()> + Send + Sync,
// {
//     fn format_record(
//         &self,
//         buf: &mut BufferFormatter,
//         msg: &LogMessage<'_>,
//     ) -> io::Result<()> {
//         (self)(buf, msg)
//     }
// }

// // enum: default (no box) + custom (boxed)
// pub enum Format {
//     Default(FormatConfig),
//     Custom(Box<dyn FormatRecord + Send + Sync>),
// }

// // unified call
// impl Format {
//     pub fn format_record(
//         &self,
//         buf: &mut BufferFormatter,
//         msg: &LogMessage<'_>,
//     ) -> io::Result<()> {
//         match self {
//             Format::Default(f) => f.format_record(buf, msg),
//             Format::Custom(f) => f.format_record(buf, msg),
//         }
//     }
// }

// // builder
// #[derive(Default)]
// pub struct FormatBuilder {
//     format_default: FormatConfig,
//     format_custom: Option<Box<dyn FormatRecord + Send + Sync>>,
// }

// impl FormatBuilder {
//     pub fn format_custom<F>(&mut self, f: F)
//     where
//         F: FormatRecord + Send + Sync + 'static,
//     {
//         self.format_custom = Some(Box::new(f));
//     }

//     pub fn build(self) -> Format {
//         if let Some(fmt) = self.format_custom {
//             Format::Custom(fmt)
//         } else {
//             Format::Default(self.format_default)
//         }
//     }
// }
