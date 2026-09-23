//! Log line level detection for the runtime log window.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

impl LogLevel {
    pub fn as_int(self) -> i32 {
        self as i32
    }

    /// Detects the level from a log line. Harness logs use a log4j-style
    /// `[time] [thread/LEVEL] [logger]: message` format; stderr lines are
    /// treated as errors when no explicit level is present.
    pub fn parse(line: &str) -> Self {
        let upper = line.to_ascii_uppercase();
        if upper.contains("FATAL") {
            Self::Fatal
        } else if upper.contains("ERROR") || upper.contains("[STDERR]") {
            Self::Error
        } else if upper.contains("WARN") {
            Self::Warn
        } else if upper.contains("DEBUG") {
            Self::Debug
        } else if upper.contains("TRACE") {
            Self::Trace
        } else {
            Self::Info
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LogLevel;

    #[test]
    fn detects_levels_from_log4j_lines() {
        assert_eq!(
            LogLevel::parse("[12:00:00] [main/INFO] [dsh]: started"),
            LogLevel::Info
        );
        assert_eq!(
            LogLevel::parse("[12:00:00] [main/WARN] [dsh]: slow"),
            LogLevel::Warn
        );
        assert_eq!(
            LogLevel::parse("[12:00:00] [main/ERROR] [dsh]: boom"),
            LogLevel::Error
        );
        assert_eq!(LogLevel::parse("[stderr] panic"), LogLevel::Error);
        assert_eq!(LogLevel::parse("plain message"), LogLevel::Info);
    }
}
