/// 用于自定义封装log日志函数
use std::{fmt::format, io::Write};
use chrono::Local;
use colored::Colorize;
use std::sync::OnceLock;

static LOGGER_CONFIG:OnceLock<LoggerConfig>=OnceLock::new();

pub enum LogLevel{
    Info,
    Warn,
    Error,
}

impl LogLevel{
    fn to_str(&self) -> String{
        match self{
            LogLevel::Info => "INFO".to_string(),
            LogLevel::Warn => "WARN".to_string(),
            LogLevel::Error => "ERROR".to_string(),
        }
    }
}

/// 日志配置
#[derive(Debug, Clone, Copy)]
pub struct LoggerConfig{
    pub enable_color: bool,
    pub show_location: bool,
    pub show_time: bool,
}

impl Default for LoggerConfig{
    fn default() -> Self {
        LoggerConfig { 
            enable_color: true, 
            show_location: true, 
            show_time: true,
        }
    }
}

pub struct Logger;

impl Logger{
    pub fn init(config: LoggerConfig){
        LOGGER_CONFIG.get_or_init(||config);
    }

    fn get_config() -> LoggerConfig{
        LOGGER_CONFIG.get().copied().unwrap_or_default()
    }

    fn format_message(level:LogLevel,color_fn: fn(&str) -> colored::ColoredString,msg:&str){
        let config = Self::get_config();
        let mut output=String::new();

        if config.show_time{
            let time =Local::now().format("%H:%M:%S");
            output.push_str(&format!("[{}]",time));
        }

        let level_str = if config.enable_color{
            format!("[{}]",color_fn(&level.to_str()).to_string())
        }else{
            format!("[{}]",level.to_str())
        };

        output.push_str(&level_str);

        output.push_str(&format!(" {}", msg));

        println!("{}", output);
    }
}

pub fn log_info(msg: &str) {
    Logger::format_message(LogLevel::Info, |s| s.blue(), msg);
}

pub fn log_warn(msg: &str) {
    Logger::format_message(LogLevel::Warn, |s| s.yellow(), msg);
}

pub fn log_error(msg: &str) {
    Logger::format_message(LogLevel::Error, |s| s.red(), msg);
}
