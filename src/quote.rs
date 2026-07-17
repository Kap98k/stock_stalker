use crate::error::{QuoteError, ParsedFieldError};

#[derive(Debug, Clone, PartialEq)]
pub struct StockQuote{
    pub ticker: String,
    pub price: f64,
    pub volume:u32,
    pub timestamp: u64
}

impl StockQuote {
    pub fn new(ticker: String, price:f64, volume: u32, timestamp: u64) ->Self {
        Self { ticker, price, volume, timestamp }
    }

    pub fn to_wire_line(&self) -> String {
        format!("{}|{}|{}|{}\n", self.ticker,self.price,self.volume, self.timestamp)
    }

    // разбор полезной нагрузки без "\n"
    pub fn from_wire_line(line: &str) -> Result<Self, QuoteError>{
        let parts:Vec<&str> = line.trim().split('|').collect();
        if parts.len() != 4{
            return Err(QuoteError::InvalidFormat("Количество блоков больше или меньше 4".into()));
        }

        if parts[0].contains(" "){
            return Err(QuoteError::InvalidQuote("Котировка содержит пробелы!".into()));
        }

        Ok(StockQuote {
            ticker: parts[0].to_string(),
            price: parts[1].parse::<f64>().map_err(|e| QuoteError::ParseError(ParsedFieldError{value: parts[1].to_string(), index:1, reason: Box::new(e)}))?,
            volume: parts[2].parse::<u32>().map_err(|e| QuoteError::ParseError(ParsedFieldError{value: parts[2].to_string(), index:2, reason: Box::new(e)}))?,
            timestamp: parts[3].parse::<u64>().map_err(|e| QuoteError::ParseError(ParsedFieldError{value: parts[3].to_string(), index:3, reason: Box::new(e)}))?
        })
    }
}