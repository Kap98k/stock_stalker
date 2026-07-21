//! Типы ошибок для проекта.
//!
//! ## Описание
//!
//! Модуль определяет типы ошибок для различных компонентов проекта:
//! - `QuoteError` - ошибки, связанные с котировками
//! - `CommandError` - ошибки, связанные с командами протокола
//! - `ParsedFieldError` - ошибки разбора отдельных полей котировки

use std::fmt;
use std::error::Error;

/// Ошибка разбора отдельного поля котировки.
///
/// ## Поля
///
/// - `value` - строковое значение поля
/// - `index` - индекс поля в котировке (0-based)
/// - `reason` - причина ошибки
///
#[derive(Debug)]
pub struct ParsedFieldError {
    /// Значение поля, которое не удалось разобрать
    pub value: String,
    /// Индекс поля в котировке (0-based)
    pub index: usize,
    /// Причина ошибки разбора
    pub reason: Box<dyn Error + Send + Sync>
}

/// Ошибки, связанные с котировками.
#[derive(Debug)]
pub enum QuoteError {
    /// Устаревшая вариант - не используется
    SomeQuotes(),
    /// Тикер не найден
    TickerNotFound(String),
    /// Неверный формат котировки
    InvalidFormat(String),
    /// Неверная котировка (например, содержит пробелы)
    InvalidQuote(String),
    /// Ошибка разбора одного из полей
    ParseError(ParsedFieldError)
}

impl fmt::Display for QuoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuoteError::TickerNotFound(ticker) => write!(f, "Запрашиваемый тикер {} не найден!", ticker),
            QuoteError::InvalidFormat(err) => write!(f, "Неверный формат котировки: {}", err),
            QuoteError::InvalidQuote(err) => write!(f, "Неверная котировка: {}", err),
            QuoteError::SomeQuotes() => write!(f, "Несколько котировок в одной строке недопустимо!"),
            QuoteError::ParseError(err) => write!(f, "Ошибка '{}' при разборе поля '{}' с индексом '{}'!", err.reason, err.value, err.index)
        }
    }
}

/// Ошибки, связанные с командами протокола.
#[derive(Debug)]
pub enum CommandError {
    /// Неверная команда
    InvalidCommand(String),
    /// Тикер не найден
    TickerNotFound(String),
    /// Неверный адрес
    InvalidAddress(String),
    /// Адрес не найден
    AddressNotFound(String),
    /// Пустой список тикеров
    EmptyTickerList,
}

impl CommandError {
    /// Преобразует ошибку в строку для отправки по сети.
    ///
    /// ## Возвращает
    ///
    /// Строка в формате `ERR <описание ошибки>\n`
    pub fn to_string(&self) -> String {
        match self {
            CommandError::TickerNotFound(ticker) => format!("ERR unknown ticker: {}\n", ticker),
            CommandError::InvalidCommand(_err) => format!("ERR invalid command\n"),
            CommandError::EmptyTickerList => format!("ERR empty ticker list\n"),
            CommandError::InvalidAddress(_err) => format!("ERR invalid udp address\n"),
            CommandError::AddressNotFound(_err) => format!("ERR udp address not found\n")
        }
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::TickerNotFound(ticker) => write!(f, "Запрашиваемый тикер {} не найден!", ticker),
            CommandError::InvalidCommand(err) => write!(f, "Неверный формат команды: {}", err),
            CommandError::EmptyTickerList => write!(f, "Пустой список котировок!"),
            CommandError::InvalidAddress(err) => write!(f, "Неверный адрес {} для отправки котировки!", err),
            CommandError::AddressNotFound(err) => write!(f, "UDP поток для адреса {} не найден!", err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── CommandError ─────────────────────────────────────────

    #[test]
    fn test_command_error_to_string_ticker_not_found() {
        let e = CommandError::TickerNotFound("SBER".to_string());
        assert_eq!(e.to_string(), "ERR unknown ticker: SBER\n");
    }

    #[test]
    fn test_command_error_to_string_invalid_command() {
        let e = CommandError::InvalidCommand("bad".to_string());
        assert_eq!(e.to_string(), "ERR invalid command\n");
    }

    #[test]
    fn test_command_error_to_string_empty_ticker_list() {
        let e = CommandError::EmptyTickerList;
        assert_eq!(e.to_string(), "ERR empty ticker list\n");
    }

    #[test]
    fn test_command_error_to_string_invalid_address() {
        let e = CommandError::InvalidAddress("x".to_string());
        assert_eq!(e.to_string(), "ERR invalid udp address\n");
    }

    #[test]
    fn test_command_error_to_string_address_not_found() {
        let e = CommandError::AddressNotFound("x".to_string());
        assert_eq!(e.to_string(), "ERR udp address not found\n");
    }

    #[test]
    fn test_command_error_display() {
        let e = CommandError::EmptyTickerList;
        assert_eq!(format!("{}", e), "Пустой список котировок!");
    }

    // ── QuoteError ───────────────────────────────────────────

    #[test]
    fn test_quote_error_display_ticker_not_found() {
        let e = QuoteError::TickerNotFound("SBER".to_string());
        assert_eq!(format!("{}", e), "Запрашиваемый тикер SBER не найден!");
    }

    #[test]
    fn test_quote_error_display_invalid_format() {
        let e = QuoteError::InvalidFormat("bad".to_string());
        assert_eq!(format!("{}", e), "Неверный формат котировки: bad");
    }

    #[test]
    fn test_quote_error_display_invalid_quote() {
        let e = QuoteError::InvalidQuote("has space".to_string());
        assert_eq!(format!("{}", e), "Неверная котировка: has space");
    }

    #[test]
    fn test_quote_error_display_some_quotes() {
        let e = QuoteError::SomeQuotes();
        assert_eq!(format!("{}", e), "Несколько котировок в одной строке недопустимо!");
    }

    #[test]
    fn test_quote_error_display_parse_error() {
        let inner = "invalid float literal".parse::<f64>().unwrap_err();
        let e = QuoteError::ParseError(ParsedFieldError {
            value: "abc".to_string(),
            index: 1,
            reason: Box::new(inner),
        });
        let displayed = format!("{}", e);
        assert!(displayed.contains("Ошибка '"));
        assert!(displayed.contains("при разборе поля 'abc' с индексом '1'"));
    }

    #[test]
    fn test_quote_error_debug() {
        let e = QuoteError::InvalidFormat("x".to_string());
        let dbg = format!("{:?}", e);
        assert!(dbg.contains("InvalidFormat"));
    }

    #[test]
    fn test_command_error_debug() {
        let e = CommandError::InvalidCommand("x".to_string());
        let dbg = format!("{:?}", e);
        assert!(dbg.contains("InvalidCommand"));
    }

    #[test]
    fn test_parsed_field_error_debug() {
        let inner = "invalid float literal".parse::<f64>().unwrap_err();
        let e = ParsedFieldError {
            value: "abc".to_string(),
            index: 2,
            reason: Box::new(inner),
        };
        let dbg = format!("{:?}", e);
        assert!(dbg.contains("abc"));
        assert!(dbg.contains("2"));
    }
}
