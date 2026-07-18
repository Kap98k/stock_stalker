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
