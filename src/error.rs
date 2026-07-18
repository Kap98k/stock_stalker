use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub struct ParsedFieldError{
    pub value: String,
    pub index: usize,
    pub reason: Box<dyn Error + Send + Sync>
}

#[derive(Debug)]
pub enum QuoteError{
    SomeQuotes(),
    TickerNotFound(String),
    InvalidFormat(String),
    InvalidQuote(String),
    ParseError(ParsedFieldError)
}

impl fmt::Display for QuoteError{
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuoteError::TickerNotFound(ticker) => write!(f,"Запрашиваемый тикер {} не найден!", ticker),
            QuoteError::InvalidFormat(err) => write!(f, "Неверный формат котировки: {}", err),
            QuoteError::InvalidQuote(err)=>write!(f, "Неверная котировка: {}", err),
            QuoteError::SomeQuotes()=> write!{f, "Несколько котировок в одной строке недопустимо!"},
            QuoteError::ParseError(err ) => write!{f, "Ошибка '{}' при разборе поля '{}' c индексом '{}'!", err.reason, err.value, err.index}
        }
    }
}

pub enum CommandError{
    InvalidCommand(String),
    TickerNotFound(String),
    InvalidAddress(String),
    AddressNotFound(String),
    EmptyTickerList,
}

impl CommandError{
    pub fn to_string(&self) -> String {
        match self {
            CommandError::TickerNotFound(ticker) => format!("ERR unknown ticker: {}\n", ticker),
            CommandError::InvalidCommand(err) => format!("ERR invalid command\n"),
            CommandError::EmptyTickerList => format!("ERR empty ticker list\n"),
            CommandError::InvalidAddress(err ) => format!("ERR invalid udp address\n"),
            CommandError::AddressNotFound(err) => format!("ERR udp address not found\n")
        }
    }
}

impl fmt::Display for CommandError{
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::TickerNotFound(ticker) => write!(f,"Запрашиваемый тикер {} не найден!", ticker),
            CommandError::InvalidCommand(err) => write!(f, "Неверный формат команды: {}", err),
            CommandError::EmptyTickerList => write!{f, "Пустой список котировок!"},
            CommandError::InvalidAddress(err ) => write!{f, "Неверный адрес {} для отправки котировки !", err},
            CommandError::AddressNotFound(err) => write!{f, "UDP поток для адреса {} не найден!", err}
        }
    }
}
