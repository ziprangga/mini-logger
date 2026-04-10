use crate::LogLevel;
use crate::Logger;
use crate::RecMessage;

fn log_reduce_size(
    logger: &Logger,
    level: LogLevel,
    target: &str,
    module: &'static str,
    msg: std::fmt::Arguments,
) {
    let mut builder = RecMessage::builder();

    builder
        .level(level)
        .target(target)
        .module(Some(module))
        .msg(msg);

    logger.rec_msg(&builder.build());
}

pub fn log_build<'a>(
    logger: &Logger,
    level: LogLevel,
    target: &str,
    module: &'static str,
    msg: std::fmt::Arguments,
) {
    log_reduce_size(logger, level, target, module, msg)
}

#[cfg(feature = "log-control")]
#[macro_export]
macro_rules! log {
    // logger + target
    (logger: $logger:expr, target: $target:expr, $lvl:expr, $($arg:tt)+) => {{
        let lvl = $lvl;
        if lvl as usize <= $crate::LogLevel::get_level() as usize {
            $crate::log_build($logger, lvl, $target, module_path!(), format_args!($($arg)+));
        }
    }};
    // logger only
    (logger: $logger:expr, $lvl:expr, $($arg:tt)+) => {
        $crate::log!(logger: $logger, target: module_path!(), $lvl, $($arg)+)
    };
    // target only
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => {
        if let Some(logger) = $crate::Logger::get() {
            $crate::log!(logger: logger, target: $target, $lvl, $($arg)+)
        }
    };
    // simple log
    ($lvl:expr, $($arg:tt)+) => {
        $crate::log!(target: module_path!(), $lvl, $($arg)+)
    };
}

#[cfg(not(feature = "log-control"))]
#[macro_export]
macro_rules! log {
    ($lvl:expr, $($arg:tt)+) => {{
        let _ = &$lvl;
        let _ = format_args!($($arg)+);
    }};
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => {{
        let _ = &$target;
        let _ = &$lvl;
        let _ = format_args!($($arg)+);
    }};
    (logger: $logger:expr, $lvl:expr, $($arg:tt)+) => {{
        let _ = &$logger;
        let _ = &$lvl;
        let _ = format_args!($($arg)+);
    }};
    (logger: $logger:expr, target: $target:expr, $lvl:expr, $($arg:tt)+) => {{
        let _ = &$logger;
        let _ = &$target;
        let _ = &$lvl;
        let _ = format_args!($($arg)+);
    }};
}

// Level-specific macros
#[macro_export]
macro_rules! error { ($($arg:tt)+) => { $crate::log!($crate::LogLevel::Error, $($arg)+) }; }
#[macro_export]
macro_rules! warn { ($($arg:tt)+) => { $crate::log!($crate::LogLevel::Warn, $($arg)+) }; }
#[macro_export]
macro_rules! info { ($($arg:tt)+) => { $crate::log!($crate::LogLevel::Info, $($arg)+) }; }
#[macro_export]
macro_rules! debug { ($($arg:tt)+) => { $crate::log!($crate::LogLevel::Debug, $($arg)+) }; }
#[macro_export]
macro_rules! trace { ($($arg:tt)+) => { $crate::log!($crate::LogLevel::Trace, $($arg)+) }; }
