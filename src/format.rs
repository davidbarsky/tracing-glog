#[cfg(feature = "ansi")]
use ansi_term::{Colour, Style};
use chrono::{DateTime, Utc};
use std::fmt;
use tracing::{Level, Metadata};

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

    #[cfg(feature = "ansi")]
    pub(crate) fn format_level(level: Level, ansi: bool) -> FmtLevel {
        FmtLevel { level, ansi }
    }

    #[cfg(not(feature = "ansi"))]
    pub(crate) fn format_level(level: Level) -> FmtLevel {
        FmtLevel { level }
    }
}

impl fmt::Display for FmtLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "ansi")]
        {
            if self.ansi {
                return match self.level {
                    Level::TRACE => write!(f, "{}", Colour::Purple.paint(Self::TRACE_STR)),
                    Level::DEBUG => write!(f, "{}", Colour::Blue.paint(Self::DEBUG_STR)),
                    Level::INFO => write!(f, "{}", Colour::Green.paint(Self::INFO_STR)),
                    Level::WARN => write!(f, "{}", Colour::Yellow.paint(Self::WARN_STR)),
                    Level::ERROR => write!(f, "{}", Colour::Red.paint(Self::ERROR_STR)),
                };
            }
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

pub(crate) struct FormatTimestamp {
    time: DateTime<Utc>,
    #[cfg(feature = "ansi")]
    pub ansi: bool,
}

impl FormatTimestamp {
    #[cfg(feature = "ansi")]
    pub(crate) fn format_time(time: DateTime<Utc>, ansi: bool) -> FormatTimestamp {
        FormatTimestamp { time, ansi }
    }

    #[cfg(not(feature = "ansi"))]
    pub(crate) fn format_time(time: DateTime<Utc>) -> FormatTimestamp {
        FormatTimestamp { time }
    }
}

impl fmt::Display for FormatTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let time = self.time.format("%m%d %H:%M:%S%.6f");

        #[cfg(feature = "ansi")]
        {
            if self.ansi {
                let dimmed = Style::new().dimmed();
                let time = format!("{}", time);
                return write!(f, "{}", dimmed.paint(time));
            }
        }
        write!(f, "{}", time)
    }
}

pub(crate) struct FormatProcessData<'a> {
    pid: u32,
    thread_name: Option<&'a str>,
    metadata: &'static Metadata<'static>,
    #[cfg(feature = "ansi")]
    pub ansi: bool,
}

impl<'a> FormatProcessData<'a> {
    #[cfg(feature = "ansi")]
    pub(crate) fn format_process_data(
        pid: u32,
        thread_name: Option<&'a str>,
        metadata: &'static Metadata<'static>,
        ansi: bool,
    ) -> Self {
        FormatProcessData {
            pid,
            thread_name,
            metadata,
            ansi,
        }
    }

    #[cfg(not(feature = "ansi"))]
    pub(crate) fn format_process_data(
        pid: u32,
        thread_name: Option<&'a str>,
        metadata: &'static Metadata<'static>,
    ) -> Self {
        FormatProcessData {
            pid,
            thread_name,
            metadata,
        }
    }
}

impl<'a> fmt::Display for FormatProcessData<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let thread_name = self.thread_name.unwrap_or("");
        let target = self.metadata.target();
        let file = self.metadata.file().unwrap_or("");
        let line = match self.metadata.line() {
            Some(line) => format!("{}", line),
            None => format!(""),
        };

        #[cfg(feature = "ansi")]
        {
            if self.ansi {
                let style = Style::new().bold();
                return write!(
                    f,
                    "{pid:>5} {thread_name} [{target}] {file}:{line}",
                    pid = self.pid,
                    thread_name = style.paint(thread_name),
                    target = style.paint(target),
                    file = style.paint(file),
                    line = style.paint(line)
                );
            }
        }

        write!(
            f,
            "{pid:>5} {thread_name} [{target}] {file}:{line}] ",
            pid = self.pid,
            thread_name = thread_name,
            target = target,
            file = file,
            line = line
        )
    }
}

pub(crate) struct FormatSpanFields<'a> {
    span_name: &'static str,
    fields: Option<&'a str>,
    #[cfg(feature = "ansi")]
    pub ansi: bool,
}

impl<'a> FormatSpanFields<'a> {
    #[cfg(feature = "ansi")]
    pub(crate) fn format_fields(
        span_name: &'static str,
        fields: Option<&'a str>,
        ansi: bool,
    ) -> Self {
        Self {
            span_name,
            fields,
            ansi,
        }
    }

    #[cfg(not(feature = "ansi"))]
    pub(crate) fn format_fields(span_name: &'static str, fields: Option<&'a str>) -> Self {
        Self { span_name, fields }
    }
}

impl<'a> fmt::Display for FormatSpanFields<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "ansi")]
        {
            if self.ansi {
                let bold = Style::new().bold();
                write!(f, "{}", bold.paint(self.span_name))?;

                let italic = Style::new().italic();
                if let Some(fields) = self.fields {
                    write!(f, "{{{}}}", italic.paint(fields))?;
                };
                return Ok(());
            };
        }

        write!(f, "{}", self.span_name)?;
        if let Some(fields) = self.fields {
            write!(f, "{{{}}}", fields)?;
        };

        Ok(())
    }
}
