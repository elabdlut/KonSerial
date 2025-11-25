// 用于自定义封装log日志函数
use env_logger::Builder;
use log::{Level, LevelFilter};
use std::io::Write;
use chrono::Local;
use colored::Colorize;

/// 日志颜色配置
#[derive(Debug, Clone, Copy)]
pub struct LoggerConfig {
    /// 是否启用彩色输出
    pub enable_color: bool,
    /// 是否显示文件位置
    pub show_location: bool,
    /// 是否显示时间
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
    /// 初始化日志系统
    /// 
    /// # 参数
    /// * `level` - 日志级别过滤器
    /// * `config` - 日志配置
    /// 
    /// # 示例
    /// ```
    /// use log::LevelFilter;
    /// let config = LoggerConfig {
    ///     enable_color: true,
    ///     show_location: true,
    ///     show_time: true,
    /// };
    /// Logger::init(LevelFilter::Debug, config);
    /// ```
    pub fn init(level: LevelFilter, config: LoggerConfig) {
        // 强制启用彩色输出
        if config.enable_color {
            colored::control::set_override(true);
        }
        
        Builder::new()
            .format(move |buf, record| {
                let level_str = record.level().to_string();
                
                // 根据日志级别着色
                let colored_level = if config.enable_color {
                    match record.level() {
                        Level::Error => level_str.red().bold(),
                        Level::Warn => level_str.yellow().bold(),
                        Level::Info => level_str.blue().bold(),
                        Level::Debug => level_str.green(),
                        Level::Trace => level_str.purple(),
                    }.to_string()
                } else {
                    level_str
                };
                
                // 构建日志输出
                let mut output = String::new();
                
                // 时间戳
                if config.show_time {
                    let time = Local::now().format("%Y-%m-%d %H:%M:%S");
                    if config.enable_color {
                        output.push_str(&format!("[{}] ", time.to_string().bright_black()));
                    } else {
                        output.push_str(&format!("[{}] ", time));
                    }
                }
                
                // 日志级别
                output.push_str(&format!("[{}] ", colored_level));
                
                // 文件位置
                if config.show_location {
                    let location = format!(
                        "{}:{}",
                        record.file().unwrap_or("unknown"),
                        record.line().unwrap_or(0)
                    );
                    if config.enable_color {
                        output.push_str(&format!("[{}] ", location.cyan()));
                    } else {
                        output.push_str(&format!("[{}] ", location));
                    }
                }
                
                // 日志消息
                output.push_str(&format!("{}", record.args()));
                
                writeln!(buf, "{}", output)
            })
            .filter_level(level)
            .init();
    }
}