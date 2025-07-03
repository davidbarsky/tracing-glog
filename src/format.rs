#[cfg(feature = "ansi")]
use nu_ansi_term::{Color, Style};
use std::fmt;
use tracing::{Level, Metadata};
use tracing_subscriber::fmt::{format::Writer, time::FormatTime};

use tracing_subscriber::fmt::time::{ChronoLocal, ChronoUtc};

pub struct FormatLevelChars {
    pub trace: &'static str,
    pub debug: &'static str,
    pub info: &'static str,
    pub warn: &'static str,
    pub error: &'static str,
}

impl FormatLevelChars {
    pub const fn const_default() -> FormatLevelChars {
        FormatLevelChars {
            trace: "T",
            debug: "D",
            info: "I",
            warn: "W",
            error: "E",
        }
    }
}

impl Default for FormatLevelChars {
    fn default() -> FormatLevelChars {
        FormatLevelChars::const_default()
    }
}

pub(crate) const DEFAULT_FORMAT_LEVEL_CHARS: FormatLevelChars = FormatLevelChars::const_default();

pub(crate) struct FmtLevel {
    pub level: Level,
    pub chars: &'static FormatLevelChars,
    #[cfg(feature = "ansi")]
    pub ansi: bool,
}

impl FmtLevel {
    pub(crate) fn format_level(
        level: Level,
        chars: &'static FormatLevelChars,
        ansi: bool,
    ) -> FmtLevel {
        #[cfg(not(feature = "ansi"))]
        let _ = ansi;
        FmtLevel {
            level,
            chars,
            #[cfg(feature = "ansi")]
            ansi,
        }
    }
}

impl fmt::Display for FmtLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let chars = self.chars;
        #[cfg(feature = "ansi")]
        if self.ansi {
            return match self.level {
                Level::TRACE => write!(f, "{}", Color::Purple.paint(chars.trace)),
                Level::DEBUG => write!(f, "{}", Color::Blue.paint(chars.debug)),
                Level::INFO => write!(f, "{}", Color::Green.paint(chars.info)),
                Level::WARN => write!(f, "{}", Color::Yellow.paint(chars.warn)),
                Level::ERROR => write!(f, "{}", Color::Red.paint(chars.error)),
            };
        }
        match self.level {
            Level::TRACE => f.pad(chars.trace),
            Level::DEBUG => f.pad(chars.debug),
            Level::INFO => f.pad(chars.info),
            Level::WARN => f.pad(chars.warn),
            Level::ERROR => f.pad(chars.error),
        }
    }
}

/// Formats the current [UTC time] using [`chrono` crate].
///
/// To format the current local time instead, use the [`LocalTime`]
/// or the [`LocalTime`] type.
///
/// [UTC time]: ChronoUtc
/// [`chrono` crate]: chrono
#[derive(Clone, Debug)]
pub struct UtcTime {
    time: ChronoUtc,
}

impl FormatTime for UtcTime {
    fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result {
        #[cfg(feature = "ansi")]
        if w.has_ansi_escapes() {
            let style = Style::new().dimmed();
            write!(w, "{}", style.prefix())?;
            self.time.format_time(w)?;
            write!(w, "{}", style.suffix())?;
            return Ok(());
        }

        self.time.format_time(w)
    }
}

impl Default for UtcTime {
    fn default() -> Self {
        let fmt_string = String::from("%m%d %H:%M:%S%.6f");
        Self {
            time: ChronoUtc::new(fmt_string),
        }
    }
}

/// Formats the current [`local time`] using [`chrono` crate].
///
/// To format the UTC time instead, use the [`UtcTime`]
/// or the [`crate::time_crate::UtcTime`] type.
///
/// [`local time`]: tracing_subscriber::fmt::time::ChronoLocal
/// [`chrono` crate]: chrono
pub struct LocalTime {
    time: ChronoLocal,
}

impl FormatTime for LocalTime {
    fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result {
        #[cfg(feature = "ansi")]
        if w.has_ansi_escapes() {
            let style = Style::new().dimmed();
            write!(w, "{}", style.prefix())?;
            self.time.format_time(w)?;
            write!(w, "{}", style.suffix())?;
            return Ok(());
        }

        self.time.format_time(w)
    }
}

impl Default for LocalTime {
    fn default() -> Self {
        let fmt_string = String::from("%m%d %H:%M:%S%.6f");
        Self {
            time: ChronoLocal::new(fmt_string),
        }
    }
}

pub(crate) struct FormatProcessData<'a> {
    pub(crate) pid: u32,
    pub(crate) thread_name: Option<&'a str>,
    pub(crate) with_thread_names: bool,
    pub(crate) metadata: &'a Metadata<'a>,
    pub(crate) with_target: bool,
    #[cfg(feature = "ansi")]
    pub(crate) ansi: bool,
}

impl<'a> fmt::Display for FormatProcessData<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let thread_name = self.thread_name;
        let target = self.metadata.target();
        let file = self.metadata.file().unwrap_or("");
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
    print_span_names: bool,
}

impl<'a> FormatSpanFields<'a> {
    pub(crate) fn format_fields(
        span_name: &'static str,
        fields: Option<&'a str>,
        ansi: bool,
        print_span_names: bool,
    ) -> Self {
        #[cfg(not(feature = "ansi"))]
        let _ = ansi;
        Self {
            span_name,
            fields,
            #[cfg(feature = "ansi")]
            ansi,
            print_span_names,
        }
    }
}

impl<'a> fmt::Display for FormatSpanFields<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "ansi")]
        if self.ansi {
            let bold = Style::new().bold();

            if self.print_span_names {
                write!(f, "{}", bold.paint(self.span_name))?;
            }

            let italic = Style::new().italic();
            if let Some(fields) = self.fields {
                if self.print_span_names {
                    write!(f, "{{{}}}", italic.paint(fields))?;
                } else {
                    write!(f, "{}", italic.paint(fields))?;
                }
            };
            return Ok(());
        }

        if self.print_span_names {
            write!(f, "{}", self.span_name)?;
        }
        if let Some(fields) = self.fields {
            if self.print_span_names {
                write!(f, "{{{}}}", fields)?;
            } else {
                write!(f, "{}", fields)?;
            }
        };

        Ok(())
    }
}
