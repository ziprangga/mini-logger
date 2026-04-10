use super::LogLevel;

#[derive(Clone, Debug)]
pub struct RecMessage<'a> {
    level: LogLevel,
    target: &'a str,
    module: Option<&'a str>,
    msg: std::fmt::Arguments<'a>,
}

impl<'a> RecMessage<'a> {
    #[inline]
    pub fn builder() -> RecMessageBuilder<'a> {
        RecMessageBuilder::new()
    }

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

impl Default for RecMessage<'_> {
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
pub struct RecMessageBuilder<'a> {
    record_msg: RecMessage<'a>,
}

impl<'a> RecMessageBuilder<'a> {
    #[inline]
    pub fn new() -> Self {
        Self {
            record_msg: RecMessage::default(),
        }
    }

    #[inline]
    pub fn level(&mut self, level: LogLevel) -> &mut Self {
        self.record_msg.level = level;
        self
    }

    #[inline]
    pub fn target(&mut self, target: &'a str) -> &mut Self {
        self.record_msg.target = target;
        self
    }

    #[inline]
    pub fn module(&mut self, module: Option<&'a str>) -> &mut Self {
        self.record_msg.module = module;
        self
    }

    #[inline]
    pub fn msg(&mut self, msg: std::fmt::Arguments<'a>) -> &mut Self {
        self.record_msg.msg = msg;
        self
    }

    #[inline]
    pub fn build(&self) -> RecMessage<'a> {
        self.record_msg.clone()
    }
}

impl Default for RecMessageBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}
