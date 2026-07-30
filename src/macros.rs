// # Internal Macro Utility
//
// This helper macro is strictly intended for internal crate operation
// inside parent logging macro expansions. It must never be invoked directly
// in downstream application code.
//
// It isolates level calculations, enforces single-evaluation rules for safety,
// and yields a `(bool, Level)` tuple to successfully route around
// local scope lifetime boundaries.
#[doc(hidden)]
#[macro_export]
macro_rules! __get_level {
    ($lvl:expr) => {{
        let lvl = $lvl;
        let write_log = lvl as usize <= $crate::Level::get_level() as usize;
        (write_log, lvl)
    }};
}

#[cfg(feature = "log-control")]
#[macro_export]
#[clippy::format_args]
macro_rules! log {
    // logger + target
    (logger: $logger:expr, target: $target:expr, $lvl:expr, $($arg:tt)+) => {{
        // Destructures the internal utility tuple to safely execute conditional filtering
        let (write_log, lvl) = $crate::__get_level!($lvl);
        if write_log {
            $crate::__private_helper::log_build($logger, lvl, $target, module_path!(), format_args!($($arg)+));
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
#[clippy::format_args]
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
#[clippy::format_args]
macro_rules! error { ($($arg:tt)+) => { $crate::log!($crate::Level::Error, $($arg)+) }; }

#[macro_export]
#[clippy::format_args]
macro_rules! warn { ($($arg:tt)+) => { $crate::log!($crate::Level::Warn, $($arg)+) }; }

#[macro_export]
#[clippy::format_args]
macro_rules! info { ($($arg:tt)+) => { $crate::log!($crate::Level::Info, $($arg)+) }; }

#[macro_export]
#[clippy::format_args]
macro_rules! debug { ($($arg:tt)+) => { $crate::log!($crate::Level::Debug, $($arg)+) }; }

#[macro_export]
#[clippy::format_args]
macro_rules! trace { ($($arg:tt)+) => { $crate::log!($crate::Level::Trace, $($arg)+) }; }
