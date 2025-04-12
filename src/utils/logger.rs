//! ロギングユーティリティモジュール
//! 
//! このモジュールには、ゲーム内のロギング機能が含まれています。
//! WebAssemblyからブラウザのコンソールへのログ出力を管理します。

use wasm_bindgen::prelude::*;
use web_sys::console;

/// ログレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// デバッグ情報（詳細な情報）
    Debug,
    /// 情報（一般的な情報）
    Info,
    /// 警告（潜在的な問題）
    Warning,
    /// エラー（実行を妨げる問題）
    Error,
}

/// ロガー構造体
#[derive(Debug, Clone)]
pub struct Logger {
    /// 最小ログレベル
    min_level: LogLevel,
    /// タグ（ログの出力元を識別するための文字列）
    tag: String,
}

impl Logger {
    /// 新しいロガーを作成
    pub fn new(tag: &str, min_level: LogLevel) -> Self {
        Self {
            min_level,
            tag: tag.to_string(),
        }
    }
    
    /// ログレベルを設定
    pub fn set_level(&mut self, level: LogLevel) {
        self.min_level = level;
    }
    
    /// 現在のログレベルを取得
    pub fn level(&self) -> LogLevel {
        self.min_level
    }
    
    /// デバッグメッセージを出力
    pub fn debug(&self, message: &str) {
        if self.min_level <= LogLevel::Debug {
            log_message(LogLevel::Debug, &self.tag, message);
        }
    }
    
    /// 情報メッセージを出力
    pub fn info(&self, message: &str) {
        if self.min_level <= LogLevel::Info {
            log_message(LogLevel::Info, &self.tag, message);
        }
    }
    
    /// 警告メッセージを出力
    pub fn warn(&self, message: &str) {
        if self.min_level <= LogLevel::Warning {
            log_message(LogLevel::Warning, &self.tag, message);
        }
    }
    
    /// エラーメッセージを出力
    pub fn error(&self, message: &str) {
        if self.min_level <= LogLevel::Error {
            log_message(LogLevel::Error, &self.tag, message);
        }
    }
    
    /// フォーマット付きのデバッグメッセージを出力
    pub fn debug_fmt(&self, args: std::fmt::Arguments) {
        if self.min_level <= LogLevel::Debug {
            self.debug(&format!("{}", args));
        }
    }
    
    /// フォーマット付きの情報メッセージを出力
    pub fn info_fmt(&self, args: std::fmt::Arguments) {
        if self.min_level <= LogLevel::Info {
            self.info(&format!("{}", args));
        }
    }
    
    /// フォーマット付きの警告メッセージを出力
    pub fn warn_fmt(&self, args: std::fmt::Arguments) {
        if self.min_level <= LogLevel::Warning {
            self.warn(&format!("{}", args));
        }
    }
    
    /// フォーマット付きのエラーメッセージを出力
    pub fn error_fmt(&self, args: std::fmt::Arguments) {
        if self.min_level <= LogLevel::Error {
            self.error(&format!("{}", args));
        }
    }
}

/// ロギング初期化
pub fn init_logging(min_level: LogLevel) {
    // wasm_loggerを初期化（実際はこれが自動的に行われるため、
    // ここでは追加の設定があれば行う）
    log::info!("ロガーが初期化されました (最小レベル: {:?})", min_level);
}

/// メッセージをログに出力
fn log_message(level: LogLevel, tag: &str, message: &str) {
    let formatted_message = format!("[{}] {}", tag, message);
    
    match level {
        LogLevel::Debug => {
            console::debug_1(&JsValue::from_str(&formatted_message));
            log::debug!("{}", formatted_message);
        }
        LogLevel::Info => {
            console::info_1(&JsValue::from_str(&formatted_message));
            log::info!("{}", formatted_message);
        }
        LogLevel::Warning => {
            console::warn_1(&JsValue::from_str(&formatted_message));
            log::warn!("{}", formatted_message);
        }
        LogLevel::Error => {
            console::error_1(&JsValue::from_str(&formatted_message));
            log::error!("{}", formatted_message);
        }
    }
}

/// デバッグログを出力するマクロ
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        web_sys::console::debug_1(&wasm_bindgen::JsValue::from_str(&format!($($arg)*)));
        log::debug!($($arg)*);
    }
}

/// 情報ログを出力するマクロ
#[macro_export]
macro_rules! info_log {
    ($($arg:tt)*) => {
        web_sys::console::info_1(&wasm_bindgen::JsValue::from_str(&format!($($arg)*)));
        log::info!($($arg)*);
    }
}

/// 警告ログを出力するマクロ
#[macro_export]
macro_rules! warn_log {
    ($($arg:tt)*) => {
        web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&format!($($arg)*)));
        log::warn!($($arg)*);
    }
}

/// エラーログを出力するマクロ
#[macro_export]
macro_rules! error_log {
    ($($arg:tt)*) => {
        web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&format!($($arg)*)));
        log::error!($($arg)*);
    }
} 