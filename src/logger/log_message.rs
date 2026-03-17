use super::LogLevel;

// pub trait MessageRecord {
//     fn build_message(&self) -> LogMessage<'static>;
// }

// impl<F> MessageRecord for F
// where
//     F: Fn() -> LogMessage<'static> + Sync + Send,
// {
//     fn build_message(&self) -> LogMessage<'static> {
//         (self)()
//     }
// }

// pub type MessageLog = Box<dyn MessageRecord + Sync + Send>;

// #[derive(Default)]
// pub struct MessageBuilder {
//     default: LogMessage<'static>,
//     custom: Option<MessageLog>,
//     built: bool,
// }

// impl MessageBuilder {
//     pub fn new(default: LogMessage<'static>) -> Self {
//         Self {
//             default,
//             custom: None,
//             built: false,
//         }
//     }

//     pub fn custom(&mut self, custom: MessageLog) -> &mut Self {
//         self.custom = Some(custom);
//         self
//     }

//     pub fn build(&mut self) -> LogMessage<'static> {
//         assert!(!self.built, "attempt to re-use consumed message builder");
//         self.built = true;

//         if let Some(custom) = self.custom.take() {
//             custom.build_message()
//         } else {
//             self.default.clone()
//         }
//     }
// }

#[derive(Clone, Debug)]
pub struct LogMessage<'a> {
    level: LogLevel,
    target: &'a str,
    module: Option<&'a str>,
    msg: std::fmt::Arguments<'a>,
}

impl<'a> LogMessage<'a> {
    #[inline]
    pub fn builder() -> LogMessageBuilder<'a> {
        LogMessageBuilder::new()
    }

    // #[inline]
    // pub fn log_config(&self) -> &LogConfig<'a> {
    //     &self.log_config
    // }

    #[inline]
    pub fn level(&self) -> LogLevel {
        self.level
    }

    #[inline]
    pub fn target(&self) -> &'a str {
        self.target
    }

    #[inline]
    pub fn module(&self) -> Option<&'a str> {
        self.module
    }

    #[inline]
    pub fn msg(&self) -> &std::fmt::Arguments<'a> {
        &self.msg
    }
}

impl Default for LogMessage<'_> {
    fn default() -> Self {
        Self {
            level: LogLevel::default(),
            target: "",
            module: None,
            msg: format_args!(""),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LogMessageBuilder<'a> {
    log_message: LogMessage<'a>,
}

impl<'a> LogMessageBuilder<'a> {
    #[inline]
    pub fn new() -> Self {
        Self {
            log_message: LogMessage::default(),
        }
    }

    // #[inline]
    // pub fn log_config(&mut self, log_config: LogConfig<'a>) -> &mut Self {
    //     self.log_message.log_config = log_config;
    //     self
    // }

    #[inline]
    pub fn level(&mut self, level: LogLevel) -> &mut Self {
        // let target = self.log_message.log_config.target();
        // self.log_message.log_config = LogConfig::builder().level(level).target(target).build();
        self.log_message.level = level;
        self
    }

    #[inline]
    pub fn target(&mut self, target: &'a str) -> &mut Self {
        self.log_message.target = target;
        self
    }

    #[inline]
    pub fn module(&mut self, module: Option<&'a str>) -> &mut Self {
        self.log_message.module = module;
        self
    }

    #[inline]
    pub fn msg(&mut self, msg: std::fmt::Arguments<'a>) -> &mut Self {
        self.log_message.msg = msg;
        self
    }

    #[inline]
    pub fn build(&self) -> LogMessage<'a> {
        self.log_message.clone()
    }
}

impl Default for LogMessageBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}
