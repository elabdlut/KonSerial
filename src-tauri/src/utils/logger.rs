/// 用于自定义封装log日志函数
use chrono::Local;
use std::sync::OnceLock;

static LOGGER_CONFIG: OnceLock<LoggerConfig> = OnceLock::new();

pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn to_str(&self) -> String {
        match self {
            LogLevel::Info => "INFO".to_string(),
            LogLevel::Warn => "WARN".to_string(),
            LogLevel::Error => "ERROR".to_string(),
        }
    }
}

/// 日志配置
#[derive(Debug, Clone, Copy)]
pub struct LoggerConfig {
    pub enable_color: bool,
    pub show_location: bool,
    pub show_time: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        LoggerConfig {
            enable_color: true,
            show_location: true,
            show_time: true,
        }
    }
}

pub struct Logger;

impl Logger {
    pub fn init(config: LoggerConfig) {
        LOGGER_CONFIG.get_or_init(|| config);
    }

    fn get_config() -> LoggerConfig {
        LOGGER_CONFIG.get().copied().unwrap_or_default()
    }

    pub fn format_message(
        level: LogLevel,
        color_fn: fn(&str) -> colored::ColoredString,
        msg: &str,
        file: &str,
        line: u32,
    ) {
        let config = Self::get_config();
        let mut output = String::new();

        if config.show_time {
            let time = Local::now().format("%H:%M:%S");
            output.push_str(&format!("[{}]", time));
        }

        let level_str = if config.enable_color {
            format!("[{}]", color_fn(&level.to_str()).to_string())
        } else {
            format!("[{}]", level.to_str())
        };

        output.push_str(&level_str);

        if config.show_location {
            output.push_str(&format!(" [{}:{}]", file, line));
        }

        output.push_str(&format!(" {}", msg));

        println!("{}", output);
    }
}

#[macro_export]
macro_rules! log_info {
    ($msg:expr) => {
        {
            use colored::Colorize;
            $crate::utils::logger::Logger::format_message(
                $crate::utils::logger::LogLevel::Info,
                |s| s.blue(),
                $msg,
                file!(),
                line!()
            )
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($msg:expr) => {
        {
            use colored::Colorize;
            $crate::utils::logger::Logger::format_message(
                $crate::utils::logger::LogLevel::Warn,
                |s| s.yellow(),
                $msg,
                file!(),
                line!()
            )
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($msg:expr) => {
        {
            use colored::Colorize;
            $crate::utils::logger::Logger::format_message(
                $crate::utils::logger::LogLevel::Error,
                |s| s.red(),
                $msg,
                file!(),
                line!()
            )
        }
    };
}
