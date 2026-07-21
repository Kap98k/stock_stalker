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
        let address = command_parts[1].trim();
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
        
        let host = &host_part[..pos].trim();
        let port_str = &port_part[1..].trim();

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

#[cfg(test)]
mod tests {
    use super::*;

    // ── check_address ────────────────────────────────────────

    #[test]
    fn test_check_address_valid() {
        assert!(check_address("127.0.0.1:8080").is_ok());
        assert!(check_address("192.168.1.1:0").is_ok());
        assert!(check_address("0.0.0.0:65535").is_ok());
    }

    #[test]
    fn test_check_address_no_port() {
        assert!(check_address("127.0.0.1").is_err());
    }

    #[test]
    fn test_check_address_invalid_ip() {
        assert!(check_address("999.999.999.999:8080").is_err());
    }

    #[test]
    fn test_check_address_invalid_port() {
        assert!(check_address("127.0.0.1:99999").is_err());
        assert!(check_address("127.0.0.1:abc").is_err());
    }

    #[test]
    fn test_check_address_empty() {
        assert!(check_address("").is_err());
    }

    // ── check_stop_command ───────────────────────────────────

    #[test]
    fn test_check_stop_command_valid() {
        let result = check_stop_command("STOP 127.0.0.1:8080");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "127.0.0.1:8080");
    }

    #[test]
    fn test_check_stop_command_with_newline() {
        let result = check_stop_command("STOP 127.0.0.1:8080\n");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "127.0.0.1:8080");
    }

    #[test]
    fn test_check_stop_command_not_stop() {
        let result = check_stop_command("STREAM 127.0.0.1:8080 SBER");
        assert!(result.is_err());
    }

    #[test]
    fn test_check_stop_command_invalid_address() {
        let result = check_stop_command("STOP bad_address");
        assert!(result.is_err());
    }

    #[test]
    fn test_check_stop_command_missing_address() {
        // splitn даст пустой хвост — упадет на check_address
        let result = check_stop_command("STOP ");
        assert!(result.is_err());
    }

    // ── check_stream_command ─────────────────────────────────

    #[test]
    fn test_check_stream_command_not_stream() {
        let result = check_stream_command("STOP 127.0.0.1:8080");
        assert!(result.is_err());
    }

    #[test]
    fn test_check_stream_command_invalid_address() {
        let result = check_stream_command("STREAM bad_addr SBER,GAZP");
        assert!(result.is_err());
    }

    // ── Subscribe ────────────────────────────────────────────

    #[test]
    fn test_subscribe_new() {
        let s = Subscribe::new("127.0.0.1:8080".to_string(), vec!["SBER".to_string()]);
        assert_eq!(s.address, "127.0.0.1:8080");
        assert_eq!(s.tickers, vec!["SBER"]);
    }
}
