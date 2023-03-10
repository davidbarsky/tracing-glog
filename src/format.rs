#[cfg(feature = "ansi")]
use crate::nu_ansi_term::{Color, Style};
use std::{ffi::OsStr, fmt, io, path::Path};
use time::{format_description::FormatItem, formatting::Formattable, OffsetDateTime};
use tracing::{Level, Metadata};
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

impl<'a> WriteAdaptor<'a> {
    pub(crate) fn new(fmt_write: &'a mut dyn fmt::Write) -> Self {
        Self { fmt_write }
    }
}

impl<'a> io::Write for WriteAdaptor<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s =
            std::str::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.fmt_write
            .write_str(s)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

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

pub(crate) struct FmtLevel {
    pub level: Level,
    #[cfg(feature = "ansi")]
    pub ansi: bool,
}

impl FmtLevel {
    const TRACE_STR: &'static str = "T";
    const DEBUG_STR: &'static str = "D";
    const INFO_STR: &'static str = "I";
    const WARN_STR: &'static str = "W";
    const ERROR_STR: &'static str = "E";

    pub(crate) fn format_level(level: Level, ansi: bool) -> FmtLevel {
        #[cfg(not(feature = "ansi"))]
        let _ = ansi;
        FmtLevel {
            level,
            #[cfg(feature = "ansi")]
            ansi,
        }
    }
}

impl fmt::Display for FmtLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "ansi")]
        if self.ansi {
            return match self.level {
                Level::TRACE => write!(f, "{}", Color::Purple.paint(Self::TRACE_STR)),
                Level::DEBUG => write!(f, "{}", Color::Blue.paint(Self::DEBUG_STR)),
                Level::INFO => write!(f, "{}", Color::Green.paint(Self::INFO_STR)),
                Level::WARN => write!(f, "{}", Color::Yellow.paint(Self::WARN_STR)),
                Level::ERROR => write!(f, "{}", Color::Red.paint(Self::ERROR_STR)),
            };
        }
        match self.level {
            Level::TRACE => f.pad(Self::TRACE_STR),
            Level::DEBUG => f.pad(Self::DEBUG_STR),
            Level::INFO => f.pad(Self::INFO_STR),
            Level::WARN => f.pad(Self::WARN_STR),
            Level::ERROR => f.pad(Self::ERROR_STR),
        }
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
pub struct LocalTime<F = Vec<FormatItem<'static>>> {
    format: F,
}

impl Default for LocalTime {
    fn default() -> Self {
        let format: Vec<FormatItem> = time::format_description::parse(
            "[month][day] [hour]:[minute]:[second].[subsecond digits:6]",
        )
        .expect("Unable to make time formatter");
        Self { format }
    }
}

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

pub(crate) struct FormatProcessData<'a> {
    pub(crate) pid: u32,
    pub(crate) thread_name: Option<&'a str>,
    pub(crate) with_thread_names: bool,
    pub(crate) metadata: &'a Metadata<'a>,
    pub(crate) with_target: bool,
    #[cfg(feature = "ansi")]
    pub(crate) ansi: bool,
    pub(crate) with_trimmed_directory: bool,
    pub(crate) with_strip_prefix: &'a Option<String>,
}

impl<'a> fmt::Display for FormatProcessData<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let thread_name = self.thread_name;
        let target = self.metadata.target();
        let file = self
            .metadata
            .file()
            .map(|f| {
                if self.with_trimmed_directory {
                    let path = Path::new(f);
                    path.file_name()
                        .map(OsStr::to_str)
                        .unwrap_or(Some(f))
                        .unwrap_or(f)
                } else if let Some(prefix) = &self.with_strip_prefix {
                    Path::new(f)
                        .strip_prefix(prefix)
                        .map(|path| path.to_str().unwrap())
                        .unwrap_or(f)
                } else {
                    f
                }
            })
            .unwrap_or("");
        let line = match self.metadata.line() {
            Some(line) => format!("{}", line),
            None => String::new(),
        };
        // write the always unstyled PID
        write!(f, " {pid:>5}", pid = self.pid)?;

        #[cfg(feature = "ansi")]
        if self.ansi {
            let style = Style::new().bold();
            // start by bolding all the expected data
            write!(f, "{}", style.prefix())?;
            if let Some(name) = thread_name {
                if self.with_thread_names {
                    write!(f, " {}", name)?
                }
            }

            if self.with_target {
                write!(f, " [{}]", target)?;
            }

            write!(f, " {file}:{line}", file = file, line = line)?;

            // end bolding
            write!(f, "{}", style.suffix())?;

            return Ok(());
        }
        if let Some(name) = thread_name {
            if self.with_thread_names {
                write!(f, " {}", name)?
            }
        }

        if self.with_target {
            write!(f, " [{}]", target)?;
        }

        write!(f, " {file}:{line}", file = file, line = line)?;
        Ok(())
    }
}

/// Docs!
pub(crate) struct FormatSpanFields<'a> {
    span_name: &'static str,
    fields: Option<&'a str>,
    #[cfg(feature = "ansi")]
    pub ansi: bool,
}

impl<'a> FormatSpanFields<'a> {
    pub(crate) fn format_fields(
        span_name: &'static str,
        fields: Option<&'a str>,
        ansi: bool,
    ) -> Self {
        #[cfg(not(feature = "ansi"))]
        let _ = ansi;
        Self {
            span_name,
            fields,
            #[cfg(feature = "ansi")]
            ansi,
        }
    }
}

impl<'a> fmt::Display for FormatSpanFields<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "ansi")]
        if self.ansi {
            let bold = Style::new().bold();
            write!(f, "{}", bold.paint(self.span_name))?;

            let italic = Style::new().italic();
            if let Some(fields) = self.fields {
                write!(f, "{{{}}}", italic.paint(fields))?;
            };
            return Ok(());
        }
        write!(f, "{}", self.span_name)?;
        if let Some(fields) = self.fields {
            write!(f, "{{{}}}", fields)?;
        };

        Ok(())
    }
}
