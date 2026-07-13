use std::fmt;
use std::error::Error;
use std::num::{ParseFloatError, ParseIntError};

#[derive(Debug)]
pub struct ParsedFieldError{
    pub value: String,
    pub index: usize,
    pub reason: Box<dyn Error + Send + Sync>
}

#[derive(Debug)]
pub enum QuoteError{
    SomeQuotes(),
    InvalidFormat(String),
    InvalidTicker(String),
    ParseError(ParsedFieldError)
}

impl fmt::Display for QuoteError{
    fn fmt(&self, f : &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuoteError::InvalidFormat(err) => write!(f, "Неверный формат котировки: {}", err),
            QuoteError::InvalidTicker(err)=>write!(f, "Неверный тикер: {}", err),
            QuoteError::SomeQuotes()=> write!{f, "Несколько котировок в одной строке недопустимо!"},
            QuoteError::ParseError(err ) => write!{f, "Ошибка '{}' при разборе поля '{}' c индексом '{}'!", err.reason, err.value, err.index}
        }
    }
}