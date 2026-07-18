use crate::error::{QuoteError, ParsedFieldError};
use crate::protocol::*;

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

use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

pub struct QuoteStream {
    socket: UdpSocket,
    subscriber: Subscribe,
    streaming: bool
}

impl QuoteStream {
    pub fn new(subscriber: Subscribe) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(&subscriber.address)?;
        Ok(Self{socket, subscriber, streaming:false})       
    }

    pub fn get_address(&self)-> String {
        self.subscriber.address.clone()
    }

    pub fn send(&self, quote: &StockQuote, addr: &str) -> Result<(), Box<dyn std::error::Error>>{
        let buf = quote.to_wire_line();
        self.socket.send_to(&buf.as_bytes(), addr)?;
        Ok(())
    }

    pub fn run_stream(&mut self, interval_ms:u64) -> Result<(), Box<dyn std::error::Error>>{
        println!("Запускаем стрим котировок для {}", &self.subscriber.address);
        self.streaming = true;

        loop{
            if self.streaming {
                let quote = Self::random(&self.subscriber.tickers);
                match self.send(&quote, &self.subscriber.address) {
                    Ok(()) => {
                        println!("Отправлена котировка: {}", quote.to_wire_line());
                    }
                    Err(e) => {
                        eprintln!("Ошибка отправки: {}", e);
                    }
                }
                thread::sleep(Duration::from_millis(interval_ms));
            }
        }
    }

    pub fn equal_address(&self, addr: &str) -> bool {
        return addr == self.subscriber.address;
    }

    // Метод для имитации метрик
    #[cfg(feature = "random")]
    pub fn random(tickers: &Vec<String>) -> StockQuote {
        use rand::{RngExt, rngs::ThreadRng};
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut rng = ThreadRng::default();
        StockQuote::new(
            tickers[rng.random_range(0..tickers.len()-1)].clone(),
            rng.random_range(100.0..5000.0),
            rng.random_range(10..1000),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        )
    }

    pub fn stop_stream(&mut self){
        if self.streaming{
            self.streaming = false;
        }
    }

    /// Запускает стриминг в отдельном потоке
    pub fn spawn_stream(mut self, interval_ms: u64) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let _ = self.run_stream(interval_ms);
        })
    }
}
