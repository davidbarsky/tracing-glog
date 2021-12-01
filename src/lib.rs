//! `tracing-glog` is a [glog]-inspired formatter for [`tracing-subscriber`].
//!
//! `tracing-glog` should be used with [`tracing-subscriber`], as it is a formatter
//! that `tracing-subscriber`'s [`fmt::Subscriber`] and [`fmt::Layer`] can use to
//! format events in a glog-inspired fashion. Here's an example:
//!
//! <pre><font color="#26A269">I</font><font color="#8D8F8A">1201 01:13:04.724801 1025672 </font><b>main</b> [<b>yak_shave</b>] <b>examples/yak-shave.rs</b>:<b>34</b>] preparing to shave yaks, <b>number_of_yaks</b>: 3
//! <font color="#26A269">I</font><font color="#8D8F8A">1201 01:13:04.724948 1025672 </font><b>main</b> [<b>yak_shave</b>] <b>examples/yak-shave.rs</b>:<b>75</b>] [<b>shaving_yaks</b>{<i><b>yaks</b></i>: 3}] shaving yaks
//! <font color="#A2734C">W</font><font color="#8D8F8A">1201 01:13:04.725071 1025672 </font><b>main</b> [<b>yak_shave</b>] <b>examples/yak-shave.rs</b>:<b>56</b>] [<b>shaving_yaks</b>{<i><b>yaks</b></i>: 3}, <b>shave</b>{<i><b>yak</b></i>: 3}] could not locate yak
//! <font color="#C01C28">E</font><font color="#8D8F8A">1201 01:13:04.725135 1025672 </font><b>main</b> [<b>yak_shave</b>] <b>examples/yak-shave.rs</b>:<b>85</b>] [<b>shaving_yaks</b>{<i><b>yaks</b></i>: 3}] failed to shave yak, <b>yak</b>: 3, <b>error</b>: out of cash
//! <font color="#26A269">I</font><font color="#8D8F8A">1201 01:13:04.725195 1025672 </font><b>main</b> [<b>yak_shave</b>] <b>examples/yak-shave.rs</b>:<b>38</b>] yak shaving completed, <b>all_yaks_shaved</b>: false
//! </pre>
//!
//! ## Examples
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
//! use tracing_glog::{Glog, GlogFields};
//! use tracing_subscriber::{fmt, Registry};
//! use tracing_subscriber::prelude::*;
//!
//! let fmt = fmt::Layer::default()
//!     .event_format(Glog::default())
//!     .fmt_fields(GlogFields::default());
//!
//! let subscriber = Registry::default().with(fmt);
//! tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");
//! ```
//!
//! [glog]: https://github.com/google/glog
//! [`tracing-subscriber`]: https://docs.rs/tracing-subscriber
//! [`fmt::Subscriber`]: tracing_subscriber::fmt::Subscriber
//! [`fmt::Layer`]: tracing_subscriber::fmt::Layer

mod format;

use ansi_term::Style;
use chrono::Utc;
use format::FmtLevel;
use std::fmt;
use tracing::{
    field::{Field, Visit},
    Subscriber,
};
use tracing_subscriber::{
    field::{MakeVisitor, VisitFmt, VisitOutput},
    fmt::{format::Writer, FmtContext, FormatEvent, FormatFields, FormattedFields},
    registry::LookupSpan,
};

use crate::format::{FormatProcessData, FormatSpanFields, FormatTimestamp};

#[derive(Default)]
pub struct Glog;

impl<S, N> FormatEvent<S, N> for Glog
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
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
        let now = Utc::now();
        let time = FormatTimestamp::format_time(now, writer.has_ansi_escapes());
        write!(writer, "{} ", time)?;

        // get some process information
        let pid = get_pid();
        let thread = std::thread::current();
        let thread_name = thread.name();

        let data = FormatProcessData::format_process_data(
            pid,
            thread_name,
            event.metadata(),
            writer.has_ansi_escapes(),
        );
        write!(writer, "{}] ", data)?;

        // now, we're printing the span context into brackets of `[]`, which glog parsers ignore.
        let leaf = ctx.lookup_current();

        if let Some(leaf) = leaf {
            // write the opening brackets
            write!(writer, "[")?;

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

                let fields =
                    FormatSpanFields::format_fields(span.name(), fields, writer.has_ansi_escapes());
                write!(writer, "{}", fields)?;

                drop(ext);
                match iter.next() {
                    // if there's more, add a space.
                    Some(next) => {
                        write!(writer, ", ")?;
                        span = next;
                    }
                    // if there's nothing there, close.
                    None => break,
                }
            }
            write!(writer, "] ")?;
        }

        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

#[derive(Default)]
pub struct GlogFields;

impl<'a> MakeVisitor<Writer<'a>> for GlogFields {
    type Visitor = GlogVisitor<'a>;

    #[inline]
    fn make_visitor(&self, target: Writer<'a>) -> Self::Visitor {
        GlogVisitor::new(target)
    }
}

#[doc(hidden)]
pub struct GlogVisitor<'a> {
    writer: Writer<'a>,
    is_empty: bool,
    style: Style,
    result: fmt::Result,
}

impl<'a> GlogVisitor<'a> {
    fn new(writer: Writer<'a>) -> Self {
        Self {
            writer,
            is_empty: true,
            style: Style::new(),
            result: Ok(()),
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
        } else {
            self.record_debug(field, &value)
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

        let bold = self.bold();
        match field.name() {
            "message" => self.write_padded(&format_args!("{}{:?}", self.style.prefix(), value,)),
            // Skip fields that are actually log metadata that have already been handled
            name if name.starts_with("log.") => self.result = Ok(()),
            name if name.starts_with("r#") => self.write_padded(&format_args!(
                "{}{}{}: {:?}",
                bold.prefix(),
                &name[2..],
                bold.infix(self.style),
                value
            )),
            name => self.write_padded(&format_args!(
                "{}{}{}: {:?}",
                bold.prefix(),
                name,
                bold.infix(self.style),
                value
            )),
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
