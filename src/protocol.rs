//! Обработка команд протокола для управления подписками на котировки.
//!
//! ## Описание
//!
//! Модуль предоставляет функции для разбора и валидации команд протокола:
//! - `check_stream_command` - разбор команды STREAM
//! - `check_stop_command` - разбор команды STOP
//! - `check_address` - валидация адреса

use std::net::IpAddr;
use std::str::FromStr;

use crate::error::{CommandError};
use crate::tickers::check_tickers;

/// Команда STREAM для подписки на котировки.
pub const STREAM_COMMAND: &str = "STREAM {} {}\n";

/// Команда STOP для остановки потока котировок.
pub const STOP_COMMAND: &str = "STOP {}\n";

/// Ключевое слово STREAM.
pub const STREAM: &str = "STREAM";

/// Ключевое слово PING.
pub const PING: &str = "PING";

/// Ключевое слово PONG.
pub const PONG: &str = "PONG";

/// Ключевое слово STOP.
pub const STOP: &str = "STOP";

/// Структура, представляющая подписку на котировки.
///
/// ## Поля
///
/// - `address` - UDP адрес для получения котировок
/// - `tickers` - список тикеров, на которые оформлена подписка
///
pub struct Subscribe {
    /// UDP адрес для получения котировок
    pub address: String,
    /// Список тикеров для подписки
    pub tickers: Vec<String>
}

impl Subscribe {
    /// Создает новую подписку.
    ///
    /// ## Параметры
    ///
    /// - `address` - UDP адрес в формате `host:port`
    /// - `tickers` - список тикеров
    ///
    /// ## Возвращает
    ///
    /// Новый экземпляр `Subscribe`
    pub fn new(address: String, tickers: Vec<String>) -> Self {
        Self { address, tickers }
    }
}

/// Проверяет и разбирает команду STOP.
///
/// ## Параметры
///
/// - `command` - строка команды в формате `STOP <address>`
///
/// ## Возвращает
///
/// - `Ok(String)` - адрес из команды
/// - `Err(CommandError)` - если команда неверная или адрес не прошел валидацию
///
/// ## Ошибки
///
/// - `CommandError::InvalidCommand` - если команда не начинается с "STOP"
/// - `CommandError::InvalidAddress` - если адрес неверный
pub fn check_stop_command(command: &str) -> Result<String, CommandError> {
    if command.starts_with(STOP) {
        let command_parts: Vec<&str> = command.splitn(2, " ").collect();
        let address = command_parts[1];
        match check_address(address) {
            Err(e) => Err(e),
            Ok(_) => Ok(address.to_string())
        }
    } else {
        Err(CommandError::InvalidCommand(command.to_string()))
    }
}

/// Проверяет и разбирает команду STREAM.
///
/// ## Параметры
///
/// - `command` - строка команды в формате `STREAM <address> <tickers>`
///
/// ## Возвращает
///
/// - `Ok(Subscribe)` - объект подписки с адресом и списком тикеров
/// - `Err(CommandError)` - если команда неверная или тикеры не прошли валидацию
///
/// ## Ошибки
///
/// - `CommandError::InvalidCommand` - если команда не начинается с "STREAM"
/// - `CommandError::InvalidAddress` - если адрес неверный
/// - `CommandError::EmptyTickerList` - если список тикеров пуст
/// - `CommandError::TickerNotFound` - если один из тикеров не найден
pub fn check_stream_command(command: &str) -> Result<Subscribe, CommandError> {
    if command.starts_with(STREAM) {
        let command_parts: Vec<&str> = command.splitn(3, " ").collect();
        let address = command_parts[1];
        match check_address(address) {
            Err(e) => Err(e),
            _ => {
                let req_tickers: Vec<&str> = command_parts[2].trim().split(',').collect();
                if req_tickers.is_empty() {
                    return Err(CommandError::EmptyTickerList)
                }

                match check_tickers(req_tickers) {
                    Err(e) => Err(e),
                    Ok(tickers) => {
                        Ok(Subscribe::new(address.to_string(), tickers))
                    }
                }
            }
        }
    } else {
        Err(CommandError::InvalidCommand(command.to_string()))
    }
}

/// Проверяет корректность адреса.
///
/// ## Параметры
///
/// - `input` - строка адреса в формате `host:port`
///
/// ## Возвращает
///
/// - `Ok(())` - если адрес валиден
/// - `Err(CommandError)` - если адрес неверный
///
/// ## Ошибки
///
/// - `CommandError::InvalidAddress` - если формат адреса неверный или IP/порт недействительны
pub fn check_address(input: &str) -> Result<(), CommandError> {
    if let Some(pos) = input.rfind(':') {
        let (host_part, port_part) = input.split_at(pos);
        
        let host = &host_part[..pos];
        let port_str = &port_part[1..];

        // Валидируем порт
        if port_str.parse::<u16>().is_ok() {
            match IpAddr::from_str(host) {
                Ok(_) => return Ok(()),
                Err(_) => return Err(CommandError::InvalidAddress(input.to_string()))
            }
        }
    }
    Err(CommandError::InvalidAddress(input.to_string()))
}
