// use ansi_term::{Colour, Style};
use chrono::Utc;
use std::fmt::{self, Write};
use tracing::{Level, Subscriber};
use tracing_core::field::Visit;
use tracing_subscriber::{
    field::{RecordFields, VisitOutput},
    fmt::{format::Writer, FmtContext, FormatEvent, FormatFields, FormattedFields},
    registry::LookupSpan,
};

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

        // Convert log level to a single character representation.
        let level = match level {
            Level::ERROR => "E",
            Level::WARN => "W",
            Level::INFO => "I",
            Level::DEBUG => "D",
            Level::TRACE => "T",
        };

        write!(writer, "{}", level)?;

        // write the timestamp:
        let now = Utc::now();
        write!(writer, "{} ", now.format("%m%d %H:%M:%S%.6f"))?;

        // get some process information
        let pid = get_pid();
        let thread_name = std::thread::current()
            .name()
            .map(|s| format!("[{}]", s))
            .unwrap_or_else(|| String::from(""));

        let file = event.metadata().file().unwrap_or_else(|| "");
        let line = event.metadata().line().unwrap();

        write!(
            writer,
            "{pid:>5} {thread_name} [{target}] {file}:{line}] ",
            pid = pid,
            thread_name = thread_name,
            target = event.metadata().target(),
            file = file,
            line = line
        )?;

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
                write!(writer, "{}", span.name())?;
                let ext = span.extensions();
                let fields = &ext
                    .get::<FormattedFields<N>>()
                    .expect("will never be `None`");

                if !fields.is_empty() {
                    write!(writer, "{{{}}}", fields)?;
                }
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
fn get_pid() -> i32 {
    nix::unistd::getpid().as_raw()
}
