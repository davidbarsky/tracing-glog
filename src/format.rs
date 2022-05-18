use ansi_term::{Colour, Style};
use chrono::{DateTime, TimeZone};
use std::fmt;
use tracing::{Level, Metadata};

pub(crate) struct FmtLevel {
    pub level: Level,
    pub ansi: bool,
}

impl FmtLevel {
    const TRACE_STR: &'static str = "T";
    const DEBUG_STR: &'static str = "D";
    const INFO_STR: &'static str = "I";
    const WARN_STR: &'static str = "W";
    const ERROR_STR: &'static str = "E";

    pub(crate) fn format_level(level: Level, ansi: bool) -> FmtLevel {
        FmtLevel { level, ansi }
    }
}

impl fmt::Display for FmtLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ansi {
            match self.level {
                Level::TRACE => write!(f, "{}", Colour::Purple.paint(Self::TRACE_STR)),
                Level::DEBUG => write!(f, "{}", Colour::Blue.paint(Self::DEBUG_STR)),
                Level::INFO => write!(f, "{}", Colour::Green.paint(Self::INFO_STR)),
                Level::WARN => write!(f, "{}", Colour::Yellow.paint(Self::WARN_STR)),
                Level::ERROR => write!(f, "{}", Colour::Red.paint(Self::ERROR_STR)),
            }
        } else {
            match self.level {
                Level::TRACE => f.pad(Self::TRACE_STR),
                Level::DEBUG => f.pad(Self::DEBUG_STR),
                Level::INFO => f.pad(Self::INFO_STR),
                Level::WARN => f.pad(Self::WARN_STR),
                Level::ERROR => f.pad(Self::ERROR_STR),
            }
        }
    }
}

pub(crate) struct FormatTimestamp<Tz: TimeZone> {
    time: DateTime<Tz>,
    pub ansi: bool,
}

impl<Tz: TimeZone> FormatTimestamp<Tz> {
    pub(crate) fn format_time(time: DateTime<Tz>, ansi: bool) -> FormatTimestamp<Tz> {
        FormatTimestamp { time, ansi }
    }
}

impl<Tz: TimeZone> fmt::Display for FormatTimestamp<Tz>
where
    Tz::Offset: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let time = self.time.format("%m%d %H:%M:%S%.6f");

        if self.ansi {
            let dimmed = Style::new().dimmed();
            let time = format!("{}", time);
            return write!(f, "{}", dimmed.paint(time));
        }

        write!(f, "{}", time)
    }
}

pub(crate) struct FormatProcessData<'a> {
    pid: u32,
    thread_name: Option<&'a str>,
    metadata: &'static Metadata<'static>,
    pub ansi: bool,
}

impl<'a> FormatProcessData<'a> {
    pub(crate) fn new(
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
}

impl<'a> fmt::Display for FormatProcessData<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let thread_name = self.thread_name.unwrap_or("");
        let target = self.metadata.target();
        let file = self.metadata.file().unwrap_or("");
        let line = match self.metadata.line() {
            Some(line) => format!("{}", line),
            None => String::new(),
        };

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
        } else {
            write!(
                f,
                "{pid:>5} {thread_name} [{target}] {file}:{line}",
                pid = self.pid,
                thread_name = thread_name,
                target = target,
                file = file,
                line = line
            )
        }
    }
}

pub(crate) struct FormatSpanFields<'a> {
    span_name: &'static str,
    fields: Option<&'a str>,
    pub ansi: bool,
}

impl<'a> FormatSpanFields<'a> {
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
}

impl<'a> fmt::Display for FormatSpanFields<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ansi {
            let bold = Style::new().bold();
            write!(f, "{}", bold.paint(self.span_name))?;

            let italic = Style::new().italic();
            if let Some(fields) = self.fields {
                write!(f, "{{{}}}", italic.paint(fields))?;
            };
            Ok(())
        } else {
            write!(f, "{}", self.span_name)?;
            if let Some(fields) = self.fields {
                write!(f, "{{{}}}", fields)?;
            };

            Ok(())
        }
    }
}
