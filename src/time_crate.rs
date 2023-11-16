#[cfg(feature = "ansi")]
use crate::nu_ansi_term::Style;
use std::{fmt, io};
use time::{format_description::FormatItem, formatting::Formattable, OffsetDateTime};
use tracing_subscriber::fmt::{format::Writer, time::FormatTime};

/// A bridge between `fmt::Write` and `io::Write`.
///
/// This is used by the timestamp formatting implementation for the `time`
/// crate and by the JSON formatter. In both cases, this is needed because
/// `tracing-subscriber`'s `FormatEvent`/`FormatTime` traits expect a
/// `fmt::Write` implementation, while `serde_json::Serializer` and `time`'s
/// `format_into` methods expect an `io::Write`.
pub(crate) struct WriteAdaptor<'a> {
    fmt_write: &'a mut dyn fmt::Write,
}

#[cfg(feature = "time")]
fn format_datetime(
    into: &mut Writer<'_>,
    now: OffsetDateTime,
    fmt: &impl Formattable,
) -> fmt::Result {
    let mut into = WriteAdaptor::new(into);
    now.format_into(&mut into, fmt)
        .map_err(|_| fmt::Error)
        .map(|_| ())
}

impl<'a> WriteAdaptor<'a> {
    pub(crate) fn new(fmt_write: &'a mut dyn fmt::Write) -> Self {
        Self { fmt_write }
    }
}

impl<'a> std::io::Write for WriteAdaptor<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = std::str::from_utf8(buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        self.fmt_write
            .write_str(s)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(s.as_bytes().len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> fmt::Debug for WriteAdaptor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("WriteAdaptor { .. }")
    }
}

/// Formats the current [UTC time] using a [formatter] from the [`time` crate].
///
/// To format the current [local time] instead, use the [`LocalTime`] type.
///
/// [UTC time]: time::OffsetDateTime::now_utc
/// [formatter]: time::formatting::Formattable
/// [`time` crate]: time
/// [local time]: time::OffsetDateTime::now_local
#[derive(Clone, Debug)]
pub struct UtcTime<F = Vec<FormatItem<'static>>> {
    format: F,
}

impl<F> FormatTime for UtcTime<F>
where
    F: Formattable,
{
    fn format_time(&self, writer: &mut Writer<'_>) -> fmt::Result {
        let now = OffsetDateTime::now_utc();

        #[cfg(feature = "ansi")]
        if writer.has_ansi_escapes() {
            let style = Style::new().dimmed();
            write!(writer, "{}", style.prefix())?;
            format_datetime(writer, now, &self.format)?;
            write!(writer, "{}", style.suffix())?;
            return Ok(());
        }

        format_datetime(writer, now, &self.format)
    }
}

impl Default for UtcTime {
    fn default() -> Self {
        let format: Vec<FormatItem> = time::format_description::parse(
            "[month][day] [hour]:[minute]:[second].[subsecond digits:6]",
        )
        .expect("Unable to make time formatter");
        Self { format }
    }
}

/// Formats the current [local time] using a [formatter] from the [`time` crate].
///
/// To format the current [UTC time] instead, use the [`UtcTime`] type.
///
/// <div class="example-wrap" style="display:inline-block">
/// <pre class="compile_fail" style="white-space:normal;font:inherit;">
///     <strong>Warning</strong>: The <a href = "https://docs.rs/time/0.3/time/"><code>time</code>
///     crate</a> must be compiled with <code>--cfg unsound_local_offset</code> in order to use
///     local timestamps. When this cfg is not enabled, local timestamps cannot be recorded, and
///     events will be logged without timestamps.
///
///    See the <a href="https://docs.rs/time/0.3.4/time/#feature-flags"><code>time</code>
///    documentation</a> for more details.
/// </pre></div>
///
/// [local time]: time::OffsetDateTime::now_local
/// [formatter]: time::formatting::Formattable
/// [`time` crate]: time
/// [UTC time]: time::OffsetDateTime::now_utc
#[derive(Clone, Debug)]
#[cfg(feature = "local-time")]
pub struct LocalTime<F = Vec<FormatItem<'static>>> {
    format: F,
}

#[cfg(feature = "local-time")]
impl Default for LocalTime {
    fn default() -> Self {
        let format: Vec<FormatItem> = time::format_description::parse(
            "[month][day] [hour]:[minute]:[second].[subsecond digits:6]",
        )
        .expect("Unable to make time formatter");
        Self { format }
    }
}

#[cfg(feature = "local-time")]
impl<F> FormatTime for LocalTime<F>
where
    F: Formattable,
{
    fn format_time(&self, writer: &mut Writer<'_>) -> fmt::Result {
        let now = OffsetDateTime::now_local().map_err(|_| fmt::Error)?;

        #[cfg(feature = "ansi")]
        if writer.has_ansi_escapes() {
            let style = Style::new().dimmed();
            write!(writer, "{}", style.prefix())?;
            format_datetime(writer, now, &self.format)?;
            // necessary to provide space between the time and the PID
            write!(writer, "{}", style.suffix())?;
            return Ok(());
        }

        format_datetime(writer, now, &self.format)
    }
}
