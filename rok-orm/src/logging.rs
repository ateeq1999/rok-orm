//! Query logging and observability.
//!
//! # Example
//!
//! ```rust,ignore
//! use rok_orm::logging::{Logger, LogLevel};
//!
//! let logger = Logger::new()
//!     .with_slow_query_threshold(100) // Log queries > 100ms
//!     .with_log_level(LogLevel::Debug);
//!
//! // Enable logging in your pool
//! ```

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn should_log(&self, level: LogLevel) -> bool {
        let self_priority = match self {
            LogLevel::Trace => 0,
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warn => 3,
            LogLevel::Error => 4,
        };
        let target_priority = match level {
            LogLevel::Trace => 0,
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warn => 3,
            LogLevel::Error => 4,
        };
        target_priority >= self_priority
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub sql: String,
    pub params_count: usize,
    pub duration_ms: u64,
    pub level: LogLevel,
    pub slow: bool,
}

impl LogEntry {
    pub fn new(sql: String, params_count: usize, duration: Duration, level: LogLevel) -> Self {
        let duration_ms = duration.as_millis() as u64;
        Self {
            sql,
            params_count,
            duration_ms,
            level,
            slow: false,
        }
    }

    pub fn with_slow_flag(mut self, threshold_ms: u64) -> Self {
        self.slow = self.duration_ms > threshold_ms;
        self
    }
}

pub struct Logger {
    log_level: LogLevel,
    slow_query_threshold_ms: u64,
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Logger {
    pub fn new() -> Self {
        Self {
            log_level: LogLevel::default(),
            slow_query_threshold_ms: 1000,
        }
    }

    pub fn with_log_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }

    pub fn with_slow_query_threshold(mut self, threshold_ms: u64) -> Self {
        self.slow_query_threshold_ms = threshold_ms;
        self
    }

    pub fn should_log(&self, level: LogLevel) -> bool {
        self.log_level.should_log(level)
    }

    pub fn is_slow_query(&self, duration_ms: u64) -> bool {
        duration_ms > self.slow_query_threshold_ms
    }

    pub fn log(&self, entry: LogEntry) {
        let level = if entry.slow {
            LogLevel::Warn
        } else {
            entry.level
        };

        if self.should_log(level) {
            self.emit_log(&entry);
        }
    }

    fn emit_log(&self, entry: &LogEntry) {
        let prefix = match entry.level {
            LogLevel::Trace => "[TRACE]",
            LogLevel::Debug => "[DEBUG]",
            LogLevel::Info => "[INFO]",
            LogLevel::Warn => "[WARN]",
            LogLevel::Error => "[ERROR]",
        };

        let slow_marker = if entry.slow { " (SLOW)" } else { "" };

        eprintln!(
            "{} Query took {}ms{}, params: {}",
            prefix, entry.duration_ms, slow_marker, entry.params_count
        );
        eprintln!("  SQL: {}", entry.sql);
    }
}

pub struct QueryTimer {
    start: Instant,
}

impl QueryTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed().as_millis() as u64
    }
}

impl Default for QueryTimer {
    fn default() -> Self {
        Self::new()
    }
}
