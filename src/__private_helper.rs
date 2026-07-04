// NOTE: This private module acts as a backend bridge for macro expansions.
// It abstracts heavy execution logic out of token streams and into standard
// compiled functions. This strictly enforces clean scoping, keeps macro
// expansion footprints as lightweight as possible, and halts binary size bloat.
#[doc(hidden)]
mod __private {
    use crate::FilterLevel;
    use crate::Logger;
    use crate::RecMessage;

    // This helper isolates the heavy builder logic into a standalone,
    // non-generic function. Because macros expand code linearly at every
    // single call site, putting this logic inside the macro would cause
    // massive binary bloat (code size explosion). Outlining this into a
    // function ensures the compiler only generates this machine code once.
    fn log_reduce_size(
        logger: &Logger,
        level: FilterLevel,
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

    // This is the public gatekeeper for internal macro expansion.
    // It should never be called manually in user code. It immediately forwards
    // execution to `log_reduce_size` to minimize the footprint of the `log!`
    // macro expansion, keeping compilation fast and binaries lean.
    pub fn log_build<'a>(
        logger: &Logger,
        level: FilterLevel,
        target: &str,
        module: &'static str,
        msg: std::fmt::Arguments,
    ) {
        log_reduce_size(logger, level, target, module, msg)
    }
}

pub use __private::log_build;
