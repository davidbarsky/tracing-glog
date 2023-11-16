#[cfg(feature = "ansi")]
use nu_ansi_term::{Color, Style};
use std::fmt;
use tracing::{Level, Metadata};
use tracing_subscriber::fmt::{format::Writer, time::FormatTime};

use tracing_subscriber::fmt::time::{ChronoLocal, ChronoUtc};

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

/// Formats the current [UTC time] using [`chrono` crate].
///
/// To format the current local time instead, use the [`ChronoLocalTime`]
/// or the [`LocalTime`] type.
///
/// [UTC time]: ChronoUtc
/// [`chrono` crate]: chrono
#[derive(Clone, Debug)]
pub struct ChronoUtcTime {
    time: ChronoUtc,
}

impl FormatTime for ChronoUtcTime {
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

impl Default for ChronoUtcTime {
    fn default() -> Self {
        let fmt_string = String::from("%m%d %H:%M:%S%.6f");
        Self {
            time: ChronoUtc::new(fmt_string),
        }
    }
}

/// Formats the current [`local time`] using [`chrono` crate].
///
/// To format the UTC time instead, use the [`ChronoUtcTime`]
/// or the [`UtcTime`] type.
///
/// [`local time`]: ChronoLocal
/// [`chrono` crate]: chrono
pub struct ChronoLocalTime {
    time: ChronoLocal,
}

impl FormatTime for ChronoLocalTime {
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

impl Default for ChronoLocalTime {
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
