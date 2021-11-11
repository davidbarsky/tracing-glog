mod format;

use ansi_term::Style;
use chrono::Utc;
use format::FmtLevel;
use std::fmt::{self, Write};
use tracing::Subscriber;
use tracing_core::field::Visit;
use tracing_subscriber::{
    field::{RecordFields, VisitOutput},
    fmt::{format::Writer, FmtContext, FormatEvent, FormatFields, FormattedFields},
    registry::LookupSpan,
};

use crate::format::{FormatProcessData, FormatSpanFields, FormatTimestamp};

pub struct GlogEventFormatter;

impl<S, N> FormatEvent<S, N> for GlogEventFormatter
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
        #[cfg(feature = "ansi")]
        let level = FmtLevel::format_level(level, writer.has_ansi_escapes());
        #[cfg(not(feature = "ansi"))]
        let level = FmtLevel::format_level(level);

        write!(writer, "{}", level)?;

        // write the timestamp:
        let now = Utc::now();
        #[cfg(feature = "ansi")]
        let time = FormatTimestamp::format_time(now, writer.has_ansi_escapes());
        #[cfg(not(feature = "ansi"))]
        let time = FormatTimestamp::format_time(now);
        write!(writer, "{} ", time)?;

        // get some process information
        let pid = get_pid();
        let thread = std::thread::current();
        let thread_name = thread.name();

        #[cfg(feature = "ansi")]
        let data = FormatProcessData::format_process_data(
            pid,
            thread_name,
            event.metadata(),
            writer.has_ansi_escapes(),
        );
        #[cfg(not(feature = "ansi"))]
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

                #[cfg(feature = "ansi")]
                let fields =
                    FormatSpanFields::format_fields(span.name(), fields, writer.has_ansi_escapes());
                #[cfg(not(feature = "ansi"))]
                let fields = FormatSpanFields::format_fields(span.name(), fields);
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

pub struct GlogFieldFormatter;

struct GlogVisitor<'a> {
    writer: &'a mut dyn Write,
}

impl<'a> GlogVisitor<'a> {
    fn new(writer: &'a mut dyn Write) -> Self {
        Self { writer }
    }
}

impl<'a> Visit for GlogVisitor<'a> {
    fn record_str(&mut self, field: &tracing_core::Field, value: &str) {
        // special-case the "message" field to elide the "message" key.
        if field.name() == "message" {
            self.record_debug(field, &format_args!("{}", value))
        } else {
            self.record_debug(field, &value)
        }
    }

    fn record_debug(&mut self, field: &tracing_core::Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            let _ = write!(self.writer, "{:?}", value);
        } else {
            #[cfg(feature = "ansi")]
            {
                let italic = Style::new().italic();
                let _ = write!(self.writer, "{}: {:?}", italic.paint(field.name()), value);
                return;
            }
            #[cfg(not(feature = "ansi"))]
            let _ = write!(self.writer, "{}: {:?}", field, value);
        }
    }
}

impl<'writer> FormatFields<'writer> for GlogFieldFormatter {
    fn format_fields<R: RecordFields>(
        &self,
        mut writer: Writer<'writer>,
        fields: R,
    ) -> fmt::Result {
        let mut visitor = GlogVisitor::new(&mut writer);
        fields.record(&mut visitor);
        visitor.finish()
    }
}

impl<'a> VisitOutput<fmt::Result> for GlogVisitor<'a> {
    fn finish(self) -> fmt::Result {
        // this is a no-op for glog
        Ok(())
    }
}

#[inline(always)]
fn get_pid() -> u32 {
    std::process::id()
}
