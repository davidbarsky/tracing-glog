//! `tracing-glog` is a [glog]-inspired formatter for [`tracing-subscriber`].
//!
//! `tracing-glog` is meant to be used with [`tracing-subscriber`]'s [`fmt::Subscriber`]
//! and [`fmt::Layer`] to format events in a `glog`-inspired fashion. Similar to
//! `tracing-subscriber`'s [`Full`] formatter, this formatter shows the span context before
//! printing event data. Spans are displayed including their names and fields. The severity,
//! time, PID, thread name, file, and line are also included.
//!
//! # Example Output
//!
//! <pre><font color="#26A269">I</font><font color="#8D8F8A">1201 01:13:04.724801 1025672 </font><b>main</b> [<b>yak_shave</b>] <b>examples/yak-shave.rs</b>:<b>34</b>] preparing to shave yaks, <b>number_of_yaks</b>: 3
//! <font color="#26A269">I</font><font color="#8D8F8A">1201 01:13:04.724948 1025672 </font><b>main</b> [<b>yak_shave</b>] <b>examples/yak-shave.rs</b>:<b>75</b>] [<b>shaving_yaks</b>{<i><b>yaks</b></i>: 3}] shaving yaks
//! <font color="#A2734C">W</font><font color="#8D8F8A">1201 01:13:04.725071 1025672 </font><b>main</b> [<b>yak_shave</b>] <b>examples/yak-shave.rs</b>:<b>56</b>] [<b>shaving_yaks</b>{<i><b>yaks</b></i>: 3}, <b>shave</b>{<i><b>yak</b></i>: 3}] could not locate yak
//! <font color="#C01C28">E</font><font color="#8D8F8A">1201 01:13:04.725135 1025672 </font><b>main</b> [<b>yak_shave</b>] <b>examples/yak-shave.rs</b>:<b>85</b>] [<b>shaving_yaks</b>{<i><b>yaks</b></i>: 3}] failed to shave yak, <b>yak</b>: 3, <b>error</b>: out of cash
//! <font color="#26A269">I</font><font color="#8D8F8A">1201 01:13:04.725195 1025672 </font><b>main</b> [<b>yak_shave</b>] <b>examples/yak-shave.rs</b>:<b>38</b>] yak shaving completed, <b>all_yaks_shaved</b>: false
//! </pre>
//!
//! # Usage
//!
//! With [`fmt::Subscriber`]:
//!
//! ```
//! use tracing_glog::{Glog, GlogFields};
//!
//! tracing_subscriber::fmt()
//!     .event_format(Glog::default())
//!     .fmt_fields(GlogFields::default())
//!     .init();
//! ```
//!
//! With [`fmt::Layer`]:
//!
//! ```
//! use tracing_subscriber::prelude::*;
//! use tracing_subscriber::{fmt, Registry};
//! use tracing_glog::{Glog, GlogFields};
//!
//! let fmt = fmt::Layer::default()
//!     .event_format(Glog::default())
//!     .fmt_fields(GlogFields::default());
//!
//! let subscriber = Registry::default().with(fmt);
//! tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");
//! ```
//!
//! With [`UtcTime`]:
//!
//! ```
//! use tracing_subscriber::prelude::*;
//! use tracing_subscriber::{fmt, Registry};
//! use tracing_glog::{Glog, GlogFields};
//!
//! let fmt = fmt::Layer::default()
//!     .event_format(Glog::default().with_timer(tracing_glog::UtcTime::default()))
//!     .fmt_fields(GlogFields::default());
//!
//! let subscriber = Registry::default().with(fmt);
//! tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");
//! ```
//!
//! With [`LocalTime`]:
//!
//! ```
//! use tracing_subscriber::prelude::*;
//! use tracing_subscriber::{fmt, Registry};
//! use tracing_glog::{Glog, GlogFields};
//!
//! let fmt = fmt::Layer::default()
//!     .event_format(Glog::default().with_timer(tracing_glog::LocalTime::default()))
//!     .fmt_fields(GlogFields::default());
//!
//! let subscriber = Registry::default().with(fmt);
//! tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");
//! ```
//!
//! <div class="example-wrap" style="display:inline-block">
//! <pre class="compile_fail" style="white-space:normal;font:inherit;">
//!     <strong>Warning</strong>: The <a href = "https://docs.rs/time/0.3/time/"><code>time</code>
//!     crate</a> must be compiled with <code>--cfg unsound_local_offset</code> in order to use
//!     `LocalTime`. When this cfg is not enabled, local timestamps cannot be recorded, and
//!     no events will be emitted.
//!
//!    See the <a href="https://docs.rs/time/0.3.9/time/#feature-flags"><code>time</code>
//!    documentation</a> for more details.
//! </pre></div>
//!
//! [glog]: https://github.com/google/glog
//! [`tracing-subscriber`]: https://docs.rs/tracing-subscriber
//! [`fmt::Subscriber`]: tracing_subscriber::fmt::Subscriber
//! [`fmt::Layer`]: tracing_subscriber::fmt::Layer
//! [`Full`]: tracing_subscriber::fmt::format::Full

#[deny(rustdoc::broken_intra_doc_links)]
mod format;

#[cfg(feature = "ansi")]
mod nu_ansi_term {
    pub use ::nu_ansi_term::*;
}
#[cfg(not(feature = "ansi"))]
mod nu_ansi_term {
    // Minimal API shim for nu_ansi_term to avoid a pile of #[cfg(feature = "ansi")] directives.
    #[derive(Copy, Clone)]
    pub struct Style;

    impl Style {
        pub fn new() -> Self {
            Style
        }
        pub fn bold(&self) -> Self {
            Style
        }
        pub fn prefix(&self) -> &'static str {
            ""
        }
        pub fn infix(&self, _: Style) -> &'static str {
            ""
        }
        pub fn suffix(&self) -> &'static str {
            ""
        }
    }
}

use crate::nu_ansi_term::Style;
use format::FmtLevel;
#[cfg(feature = "chrono")]
pub use format::{ChronoLocalTime, ChronoUtcTime};
pub use format::{LocalTime, UtcTime};
use std::fmt;
use tracing::{
    field::{Field, Visit},
    Subscriber,
};
#[cfg(feature = "tracing-log")]
use tracing_log::NormalizeEvent;
use tracing_subscriber::{
    field::{MakeVisitor, VisitFmt, VisitOutput},
    fmt::{
        format::Writer, time::FormatTime, FmtContext, FormatEvent, FormatFields, FormattedFields,
    },
    registry::LookupSpan,
};

use crate::format::{FormatProcessData, FormatSpanFields};

/// A [glog]-inspired span and event formatter.
///
/// [glog]: https://github.com/google/glog
pub struct Glog<T = UtcTime> {
    timer: T,
    with_span_context: bool,
    with_thread_names: bool,
    with_target: bool,
    with_span_names: bool,
}

impl<T> Glog<T> {
    /// Use the given [timer] for span and event time stamps.
    ///
    /// `tracing-glog` provides two timers: [`LocalTime`] and [`UtcTime`].
    /// [`UtcTime`] is the default timer.
    ///
    /// [timer]: tracing_subscriber::fmt::time::FormatTime
    pub fn with_timer<T2>(self, timer: T2) -> Glog<T2>
    where
        T2: FormatTime,
    {
        Glog {
            timer,
            with_thread_names: self.with_thread_names,
            with_target: self.with_target,
            with_span_context: self.with_span_context,
            with_span_names: self.with_span_names,
        }
    }

    pub fn with_thread_names(self, with_thread_names: bool) -> Glog<T> {
        Glog {
            with_thread_names,
            ..self
        }
    }

    pub fn with_target(self, with_target: bool) -> Glog<T> {
        Glog {
            with_target,
            ..self
        }
    }

    /// Sets whether or not the span name is included. Defaults to true.
    ///
    /// If span names are not included, then the fields from all spans are
    /// printed as a single list of fields. This results is a more compact output.
    ///
    /// # Example Output
    /// With `with_span_names` set to true:
    /// <pre>
    /// I0731 16:23:45.674465 990039 examples/tokio.rs:38] [parent_task{subtasks: 10, reason: "testing"}, subtask{number: 10}] polling subtask, number: 10
    /// </pre>
    ///
    /// With `with_span_names` set to false:
    /// <pre>
    /// I0731 16:23:45.674465 990039 examples/tokio.rs:38] [subtasks: 10, reason: "testing", number: 10] polling subtask, number: 10
    /// </pre>
    pub fn with_span_names(self, with_span_names: bool) -> Glog<T> {
        Glog {
            with_span_names,
            ..self
        }
    }

    /// Sets whether or not the span context is included. Defaults to true.
    ///
    /// By default, formatters building atop of [`mod@tracing_subscriber::fmt`]
    /// will include the span context as [`fmt::Layer`] and
    /// [`fmt::Subscriber`] assume that span context is
    /// is valuable and the _raison d’être_ for using `tracing` and, as such, do not provide a
    /// toggle. However, users migrating from logging systems such
    /// as [glog](https://github.com/google/glog) or folly's [`xlog`](https://github.com/facebook/folly/blob/main/folly/logging/xlog.h)
    /// might find the span context to be overwhelming. Therefore, this formatter-level toggle
    /// is availible in order to provide a smoother onboarding experience to [`tracing`].
    ///
    /// **Notice:** This is a relatively coarse toggle. In most circumstances, usage of
    /// `tracing-subscriber`'s [`filter_fn`] is preferred to disable spans on a more
    /// fine-grained basis.
    ///
    /// [`fmt::Layer`]: tracing_subscriber::fmt::Layer
    /// [`fmt::Subscriber`]: tracing_subscriber::fmt::Subscriber
    /// [per-layer filtering]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/index.html#per-layer-filtering
    /// [`filter_fn`]: fn@tracing_subscriber::filter::filter_fn
    /// [`tracing`]: mod@tracing
    pub fn with_span_context(self, with_span_context: bool) -> Glog<T> {
        Glog {
            with_span_context,
            ..self
        }
    }
}

impl Default for Glog<UtcTime> {
    fn default() -> Self {
        Glog {
            timer: UtcTime::default(),
            with_thread_names: false,
            with_target: false,
            with_span_context: true,
            with_span_names: true,
        }
    }
}

impl<S, N, T> FormatEvent<S, N> for Glog<T>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
    T: FormatTime,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> fmt::Result {
        let level = *event.metadata().level();

        // Convert log level to a single character representation.)
        let level = FmtLevel::format_level(level, writer.has_ansi_escapes());
        write!(writer, "{}", level)?;

        // write the timestamp:
        self.timer.format_time(&mut writer)?;

        // get some process information
        let pid = get_pid();
        let thread = std::thread::current();
        let thread_name = thread.name();

        #[cfg(feature = "tracing-log")]
        let normalized_meta = event.normalized_metadata();
        #[cfg(feature = "tracing-log")]
        let metadata = normalized_meta.as_ref().unwrap_or_else(|| event.metadata());
        #[cfg(not(feature = "tracing-log"))]
        let metadata = event.metadata();

        let data = FormatProcessData {
            pid,
            thread_name,
            with_thread_names: self.with_thread_names,
            metadata,
            with_target: self.with_target,
            #[cfg(feature = "ansi")]
            ansi: writer.has_ansi_escapes(),
        };
        write!(writer, "{}] ", data)?;

        if self.with_span_context {
            // now, we're printing the span context into brackets of `[]`, which glog parsers ignore.
            let leaf = ctx.lookup_current();

            if let Some(leaf) = leaf {
                let mut wrote_open_bracket = false;

                // Write spans and fields of each span
                let mut iter = leaf.scope().from_root();
                let mut span = iter.next().expect(
                    "Unable to get the next item in the iterator; this should not be possible.",
                );
                loop {
                    let ext = span.extensions();
                    let fields = &ext
                        .get::<FormattedFields<N>>()
                        .expect("will never be `None`");

                    let fields = if !fields.is_empty() {
                        Some(fields.as_str())
                    } else {
                        None
                    };

                    if self.with_span_names || fields.is_some() {
                        if !wrote_open_bracket {
                            // Write the opening bracket once we know we need one
                            write!(writer, "[")?;
                            wrote_open_bracket = true;
                        }
                        let fields = FormatSpanFields::format_fields(
                            span.name(),
                            fields,
                            writer.has_ansi_escapes(),
                            self.with_span_names,
                        );
                        write!(writer, "{}", fields)?;
                    }

                    drop(ext);
                    match iter.next() {
                        // if there's more, add a space.
                        Some(next) => {
                            if wrote_open_bracket {
                                write!(writer, ", ")?;
                            }
                            span = next;
                        }
                        // if there's nothing there, close.
                        None => break,
                    }
                }
                if wrote_open_bracket {
                    write!(writer, "] ")?;
                }
            }
        }
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

#[derive(Clone)]
struct FieldConfig {
    should_quote_strings: bool,
    use_whitespace_in_field: bool,
}

impl Default for FieldConfig {
    fn default() -> Self {
        Self {
            should_quote_strings: true,
            use_whitespace_in_field: true,
        }
    }
}

#[derive(Default)]
pub struct GlogFields {
    config: FieldConfig,
}

impl GlogFields {
    /// Sets whether or not strings are wrapped in quotes.
    ///
    /// This is helpful for reducing line width at the cost of clarity when
    /// using strings with whitespace as fields on Spans and Events.
    pub fn should_quote_strings(mut self, value: bool) -> Self {
        self.config.should_quote_strings = value;
        self
    }

    /// Sets whether or not whitespace is added to printed fields.
    ///
    /// This defaults to `false`.
    pub fn use_whitespace_in_field(mut self, value: bool) -> Self {
        self.config.use_whitespace_in_field = value;
        self
    }

    /// Sets the formatter to use compact options.
    ///
    /// Setting `.compat()` will set [`GlogFields::use_whitespace_in_field`]
    /// and [`GlogFields::should_quote_strings`] to false.
    pub fn compact(self) -> Self {
        self.should_quote_strings(false)
            .use_whitespace_in_field(false)
    }
}

impl<'a> MakeVisitor<Writer<'a>> for GlogFields {
    type Visitor = GlogVisitor<'a>;

    #[inline]
    fn make_visitor(&self, target: Writer<'a>) -> Self::Visitor {
        GlogVisitor::new(target, self.config.clone())
    }
}

#[doc(hidden)]
pub struct GlogVisitor<'a> {
    writer: Writer<'a>,
    is_empty: bool,
    style: Style,
    result: fmt::Result,
    config: FieldConfig,
}

impl<'a> GlogVisitor<'a> {
    fn new(writer: Writer<'a>, config: FieldConfig) -> Self {
        Self {
            writer,
            is_empty: true,
            style: Style::new(),
            result: Ok(()),
            config,
        }
    }

    fn write_padded(&mut self, value: &impl fmt::Debug) {
        let padding = if self.is_empty {
            self.is_empty = false;
            ""
        } else {
            ", "
        };
        self.result = write!(self.writer, "{}{:?}", padding, value);
    }

    fn write_field(&mut self, name: &str, value: &dyn fmt::Debug) {
        let bold = self.bold();
        if self.config.use_whitespace_in_field {
            self.write_padded(&format_args!(
                "{}{}{}: {:?}",
                bold.prefix(),
                name,
                bold.infix(self.style),
                value,
            ));
        } else {
            self.write_padded(&format_args!(
                "{}{}{}:{:?}",
                bold.prefix(),
                name,
                bold.infix(self.style),
                value,
            ));
        }
    }

    fn bold(&self) -> Style {
        if self.writer.has_ansi_escapes() {
            self.style.bold()
        } else {
            Style::new()
        }
    }
}

impl<'a> Visit for GlogVisitor<'a> {
    fn record_str(&mut self, field: &Field, value: &str) {
        if self.result.is_err() {
            return;
        }

        if field.name() == "message" {
            self.record_debug(field, &format_args!("{}", value))
        } else if self.config.should_quote_strings {
            self.record_debug(field, &value)
        } else {
            self.record_debug(field, &format_args!("{}", value))
        }
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        if let Some(source) = value.source() {
            self.record_debug(
                field,
                &format_args!("{}, {}.sources: {}", value, field, ErrorSourceList(source),),
            )
        } else {
            self.record_debug(field, &format_args!("{}", value))
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if self.result.is_err() {
            return;
        }

        match field.name() {
            "message" => self.write_padded(&format_args!("{}{:?}", self.style.prefix(), value,)),
            // Skip fields that are actually log metadata that have already been handled
            name if name.starts_with("log.") => self.result = Ok(()),
            name if name.starts_with("r#") => self.write_field(&name[2..], value),
            name => self.write_field(name, value),
        };
    }
}

impl<'a> VisitOutput<fmt::Result> for GlogVisitor<'a> {
    fn finish(mut self) -> fmt::Result {
        write!(&mut self.writer, "{}", self.style.suffix())?;
        self.result
    }
}

impl<'a> VisitFmt for GlogVisitor<'a> {
    fn writer(&mut self) -> &mut dyn fmt::Write {
        &mut self.writer
    }
}

/// Renders an error into a list of sources, *including* the error
struct ErrorSourceList<'a>(&'a (dyn std::error::Error + 'static));

impl<'a> fmt::Display for ErrorSourceList<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        let mut curr = Some(self.0);
        while let Some(curr_err) = curr {
            list.entry(&format_args!("{}", curr_err));
            curr = curr_err.source();
        }
        list.finish()
    }
}

#[inline(always)]
fn get_pid() -> u32 {
    std::process::id()
}
