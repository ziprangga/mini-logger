// #[derive(Default)]
// pub struct Builder {
//     filter: env_filter::Builder,
//     writer: writer::Builder,
//     format: fmt::Builder,
//     built: bool,
// }

// impl Builder {
//     /// Initializes the log builder with defaults.
//     ///
//     /// **NOTE:** This method won't read from any environment variables.
//     /// Use the [`filter`] and [`write_style`] methods to configure the builder
//     /// or use [`from_env`] or [`from_default_env`] instead.
//     ///
//     /// # Examples
//     ///
//     /// Create a new builder and configure filters and style:
//     ///
//     /// ```
//     /// use log::LevelFilter;
//     /// use env_logger::{Builder, WriteStyle};
//     ///
//     /// let mut builder = Builder::new();
//     ///
//     /// builder
//     ///     .filter(None, LevelFilter::Info)
//     ///     .write_style(WriteStyle::Always)
//     ///     .init();
//     /// ```
//     ///
//     /// [`filter`]: #method.filter
//     /// [`write_style`]: #method.write_style
//     /// [`from_env`]: #method.from_env
//     /// [`from_default_env`]: #method.from_default_env
//     pub fn new() -> Builder {
//         Default::default()
//     }

//     /// Initializes the log builder from the environment.
//     ///
//     /// The variables used to read configuration from can be tweaked before
//     /// passing in.
//     ///
//     /// # Examples
//     ///
//     /// Initialise a logger reading the log filter from an environment variable
//     /// called `MY_LOG`:
//     ///
//     /// ```
//     /// use env_logger::Builder;
//     ///
//     /// let mut builder = Builder::from_env("MY_LOG");
//     /// builder.init();
//     /// ```
//     ///
//     /// Initialise a logger using the `MY_LOG` variable for filtering and
//     /// `MY_LOG_STYLE` for whether or not to write styles:
//     ///
//     /// ```
//     /// use env_logger::{Builder, Env};
//     ///
//     /// let env = Env::new().filter("MY_LOG").write_style("MY_LOG_STYLE");
//     ///
//     /// let mut builder = Builder::from_env(env);
//     /// builder.init();
//     /// ```
//     pub fn from_env<'a, E>(env: E) -> Self
//     where
//         E: Into<Env<'a>>,
//     {
//         let mut builder = Builder::new();
//         builder.parse_env(env);
//         builder
//     }

//     /// Applies the configuration from the environment.
//     ///
//     /// This function allows a builder to be configured with default parameters,
//     /// to be then overridden by the environment.
//     ///
//     /// # Examples
//     ///
//     /// Initialise a logger with filter level `Off`, then override the log
//     /// filter from an environment variable called `MY_LOG`:
//     ///
//     /// ```
//     /// use log::LevelFilter;
//     /// use env_logger::Builder;
//     ///
//     /// let mut builder = Builder::new();
//     ///
//     /// builder.filter_level(LevelFilter::Off);
//     /// builder.parse_env("MY_LOG");
//     /// builder.init();
//     /// ```
//     ///
//     /// Initialise a logger with filter level `Off`, then use the `MY_LOG`
//     /// variable to override filtering and `MY_LOG_STYLE` to override  whether
//     /// or not to write styles:
//     ///
//     /// ```
//     /// use log::LevelFilter;
//     /// use env_logger::{Builder, Env};
//     ///
//     /// let env = Env::new().filter("MY_LOG").write_style("MY_LOG_STYLE");
//     ///
//     /// let mut builder = Builder::new();
//     /// builder.filter_level(LevelFilter::Off);
//     /// builder.parse_env(env);
//     /// builder.init();
//     /// ```
//     pub fn parse_env<'a, E>(&mut self, env: E) -> &mut Self
//     where
//         E: Into<Env<'a>>,
//     {
//         let env = env.into();

//         if let Some(s) = env.get_filter() {
//             self.parse_filters(&s);
//         }

//         if let Some(s) = env.get_write_style() {
//             self.parse_write_style(&s);
//         }

//         self
//     }

//     /// Initializes the log builder from the environment using default variable names.
//     ///
//     /// This method is a convenient way to call `from_env(Env::default())` without
//     /// having to use the `Env` type explicitly. The builder will use the
//     /// [default environment variables].
//     ///
//     /// # Examples
//     ///
//     /// Initialise a logger using the default environment variables:
//     ///
//     /// ```
//     /// use env_logger::Builder;
//     ///
//     /// let mut builder = Builder::from_default_env();
//     /// builder.init();
//     /// ```
//     ///
//     /// [default environment variables]: struct.Env.html#default-environment-variables
//     pub fn from_default_env() -> Self {
//         Self::from_env(Env::default())
//     }

//     /// Applies the configuration from the environment using default variable names.
//     ///
//     /// This method is a convenient way to call `parse_env(Env::default())` without
//     /// having to use the `Env` type explicitly. The builder will use the
//     /// [default environment variables].
//     ///
//     /// # Examples
//     ///
//     /// Initialise a logger with filter level `Off`, then configure it using the
//     /// default environment variables:
//     ///
//     /// ```
//     /// use log::LevelFilter;
//     /// use env_logger::Builder;
//     ///
//     /// let mut builder = Builder::new();
//     /// builder.filter_level(LevelFilter::Off);
//     /// builder.parse_default_env();
//     /// builder.init();
//     /// ```
//     ///
//     /// [default environment variables]: struct.Env.html#default-environment-variables
//     pub fn parse_default_env(&mut self) -> &mut Self {
//         self.parse_env(Env::default())
//     }

//     pub struct Logger {
//         writer: Writer,
//         filter: env_filter::Filter,
//         format: FormatFn,
//     }

//     impl Logger {
//         /// Creates the logger from the environment.
//         ///
//         /// The variables used to read configuration from can be tweaked before
//         /// passing in.
//         ///
//         /// # Examples
//         ///
//         /// Create a logger reading the log filter from an environment variable
//         /// called `MY_LOG`:
//         ///
//         /// ```
//         /// use env_logger::Logger;
//         ///
//         /// let logger = Logger::from_env("MY_LOG");
//         /// ```
//         ///
//         /// Create a logger using the `MY_LOG` variable for filtering and
//         /// `MY_LOG_STYLE` for whether or not to write styles:
//         ///
//         /// ```
//         /// use env_logger::{Logger, Env};
//         ///
//         /// let env = Env::new().filter_or("MY_LOG", "info").write_style_or("MY_LOG_STYLE", "always");
//         ///
//         /// let logger = Logger::from_env(env);
//         /// ```
//         pub fn from_env<'a, E>(env: E) -> Self
//         where
//             E: Into<Env<'a>>,
//         {
//             Builder::from_env(env).build()
//         }

//         /// Creates the logger from the environment using default variable names.
//         ///
//         /// This method is a convenient way to call `from_env(Env::default())` without
//         /// having to use the `Env` type explicitly. The logger will use the
//         /// [default environment variables].
//         ///
//         /// # Examples
//         ///
//         /// Creates a logger using the default environment variables:
//         ///
//         /// ```
//         /// use env_logger::Logger;
//         ///
//         /// let logger = Logger::from_default_env();
//         /// ```
//         ///
//         /// [default environment variables]: struct.Env.html#default-environment-variables
//         pub fn from_default_env() -> Self {
//             Builder::from_default_env().build()
//         }

//         // filter.rs
//         use std::env;

//         impl FilterBuilder {
//             /// Parse filter directives from an environment variable.
//             pub fn parse_env_var(&mut self, var_name: &str) -> &mut Self {
//                 if let Ok(env_val) = env::var(var_name) {
//                     self.parse_filter_string(&env_val);
//                 }
//                 self
//             }

//             /// Parse a filter string like "module1=info,module2=warn"
//             pub fn parse_filter_string(&mut self, s: &str) -> &mut Self {
//                 for directive in s.split(',') {
//                     let mut parts = directive.split('=');
//                     let module = parts.next().unwrap_or("").trim();
//                     let level_str = parts.next().unwrap_or("off").trim();

//                     let level = level_str.parse::<Level>().unwrap_or(Level::Off);

//                     if module.is_empty() {
//                         self.filter_level(level);
//                     } else {
//                         self.filter_module(module, level);
//                     }
//                 }
//                 self
//             }
//         }
