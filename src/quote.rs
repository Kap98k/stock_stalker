//! Структуры для представления и обработки котировок акций.
//!
//! ## Описание
//!
//! Модуль предоставляет типы и функции для работы с котировками:
//! - `StockQuote` - структура для хранения данных о котировке
//! - `QuoteStream` - структура для управления UDP потоком котировок

use crate::error::{QuoteError, ParsedFieldError};
use crate::protocol::*;

/// Структура, представляющая котировку акции.
///
/// ## Поля
///
/// - `ticker` - тикер акции (например: "SBER", "GAZP")
/// - `price` - текущая цена акции
/// - `volume` - объем торгов
/// - `timestamp` - временная метка котировки (в секундах с начала эпохи Unix)
///
/// ## Формат передачи по сети
///
/// Котировки передаются в текстовом формате:
/// ```text
/// <TICKER>|<PRICE>|<VOLUME>|<TIMESTAMP>
/// ```
///
#[derive(Debug, Clone, PartialEq)]
pub struct StockQuote {
    /// Тикер акции
    pub ticker: String,
    /// Цена акции
    pub price: f64,
    /// Объем торгов
    pub volume: u32,
    /// Временная метка котировки (секунды с начала эпохи Unix)
    pub timestamp: u64
}

impl StockQuote {
    /// Создает новую котировку.
    ///
    /// ## Параметры
    ///
    /// - `ticker` - тикер акции
    /// - `price` - цена акции
    /// - `volume` - объем торгов
    /// - `timestamp` - временная метка котировки
    ///
    /// ## Возвращает
    ///
    /// Новую экземпляр `StockQuote`
    pub fn new(ticker: String, price: f64, volume: u32, timestamp: u64) -> Self {
        Self { ticker, price, volume, timestamp }
    }

    /// Преобразует котировку в строку для передачи по сети.
    ///
    /// ## Формат
    ///
    /// Возвращает строку в формате: `<TICKER>|<PRICE>|<VOLUME>|<TIMESTAMP>\n`
    ///
    /// ## Возвращает
    ///
    /// Строка, готовая для отправки по сети
    pub fn to_wire_line(&self) -> String {
        format!("{}|{}|{}|{}\n", self.ticker, self.price, self.volume, self.timestamp).to_string()
    }

    /// Разбирает строку котировки из сети.
    ///
    /// ## Параметры
    ///
    /// - `line` - строка в формате `<TICKER>|<PRICE>|<VOLUME>|<TIMESTAMP>`
    ///
    /// ## Возвращает
    ///
    /// - `Ok(StockQuote)` - если разбор прошел успешно
    /// - `Err(QuoteError)` - если строка имеет неверный формат или содержит ошибки
    ///
    /// ## Ошибки
    ///
    /// - `QuoteError::InvalidFormat` - если количество полей не равно 4
    /// - `QuoteError::InvalidQuote` - если тикер содержит пробелы
    /// - `QuoteError::ParseError` - если не удалось разобрать одно из числовых полей
    pub fn from_wire_line(line: &str) -> Result<Self, QuoteError> {
        let parts: Vec<&str> = line.trim().split('|').collect();
        if parts.len() != 4 {
            return Err(QuoteError::InvalidFormat("Количество блоков больше или меньше 4".into()));
        }

        if parts[0].contains(' ') {
            return Err(QuoteError::InvalidQuote("Котировка содержит пробелы!".into()));
        }

        Ok(StockQuote {
            ticker: parts[0].to_string(),
            price: parts[1].parse::<f64>().map_err(|e| QuoteError::ParseError(ParsedFieldError {
                value: parts[1].to_string(),
                index: 1,
                reason: Box::new(e)
            }))?,
            volume: parts[2].parse::<u32>().map_err(|e| QuoteError::ParseError(ParsedFieldError {
                value: parts[2].to_string(),
                index: 2,
                reason: Box::new(e)
            }))?,
            timestamp: parts[3].parse::<u64>().map_err(|e| QuoteError::ParseError(ParsedFieldError {
                value: parts[3].to_string(),
                index: 3,
                reason: Box::new(e)
            }))?
        })
    }
}

use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// Управляет UDP потоком для отправки котировок подписчику.
///
/// ## Описание
///
/// Эта структура инкапсулирует UDP сокет и информацию о подписчике.
/// Она предоставляет методы для отправки котировок и управления потоком.
///
pub struct QuoteStream {
    socket: UdpSocket,
    subscriber: Subscribe,
    streaming: Arc<AtomicBool>,
}

impl QuoteStream {
    /// Создает новый поток котировок для указанной подписки.
    ///
    /// ## Параметры
    ///
    /// - `subscriber` - информация о подписчике (адрес и список тикеров)
    ///
    /// ## Возвращает
    ///
    /// - `Ok(QuoteStream)` - если сокет успешно привязан
    /// - `Err(std::io::Error)` - если не удалось привязать сокет
    pub fn new(subscriber: Subscribe) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        Ok(Self { socket, subscriber, streaming: Arc::new(AtomicBool::new(false)) })
    }

    /// Возвращает клон атомарного флага `streaming`.
    ///
    /// Используется для внешней остановки потока без захвата Mutex.
    pub fn streaming_flag(&self) -> Arc<AtomicBool> {
        self.streaming.clone()
    }

    /// Возвращает UDP адрес подписчика.
    ///
    /// ## Возвращает
    ///
    /// Клон строки с адресом подписчика
    pub fn get_address(&self) -> String {
        self.subscriber.address.clone()
    }

    /// Отправляет котировку по указанному адресу.
    ///
    /// ## Параметры
    ///
    /// - `quote` - котировка для отправки
    /// - `addr` - адрес получателя в формате `host:port`
    ///
    /// ## Возвращает
    ///
    /// - `Ok(())` - если котировка успешно отправлена
    /// - `Err(Box<dyn std::error::Error>)` - если произошла ошибка при отправке
    pub fn send(&self, quote: &StockQuote, addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let buf = quote.to_wire_line();
        self.socket.send_to(&buf.as_bytes(), addr)?;
        Ok(())
    }

    /// Запускает бесконечный цикл отправки случайных котировок.
    ///
    /// ## Параметры
    ///
    /// - `interval_ms` - интервал между отправками в миллисекундах
    ///
    /// ## Возвращает
    ///
    /// - `Ok(())` — если цикл остановлен через `stop_stream()`
    /// - `Err(Box<dyn std::error::Error>)` — если произошла ошибка при отправке
    ///
    /// ## Примечание
    ///
    /// Этот метод можно вызывать через `Arc`, блокировка Mutex не требуется.
    pub fn run_stream(&self, interval_ms: u64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Запускаем стрим котировок для {}", &self.subscriber.address);
        self.streaming.store(true, Ordering::Relaxed);

        loop {
            if !self.streaming.load(Ordering::Relaxed) {
                return Ok(());
            }
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

    /// Проверяет, соответствует ли указанный адрес адресу подписчика.
    ///
    /// ## Параметры
    ///
    /// - `addr` - адрес для сравнения
    ///
    /// ## Возвращает
    ///
    /// `true` если адреса совпадают, иначе `false`
    pub fn equal_address(&self, addr: &str) -> bool {
        addr == self.subscriber.address
    }

    /// Генерирует случайную котировку из списка тикеров (только с фичей "random").
    ///
    /// ## Параметры
    ///
    /// - `tickers` - список доступных тикеров
    ///
    /// ## Возвращает
    ///
    /// `StockQuote` со случайными значениями цены, объема и текущего времени
    #[cfg(feature = "random")]
    pub fn random(tickers: &Vec<String>) -> StockQuote {
        use rand::{RngExt, rngs::ThreadRng};
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut rng = ThreadRng::default();
        StockQuote::new(
            tickers[rng.random_range(0..tickers.len())].clone(),
            rng.random_range(100.0..5000.0),
            rng.random_range(10..1000),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        )
    }

    /// Останавливает поток котировок.
    ///
    /// Устанавливает атомарный флаг `streaming` в `false`, что приводит
    /// к выходу из `run_stream()`. Потокобезопасен, не требует Mutex.
    pub fn stop_stream(&self) {
        self.streaming.store(false, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::UdpSocket;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_stock_quote_new() {
        let quote = StockQuote::new("SBER".to_string(), 350.50, 1000, 1689678900);
        assert_eq!(quote.ticker, "SBER");
        assert_eq!(quote.price, 350.50);
        assert_eq!(quote.volume, 1000);
        assert_eq!(quote.timestamp, 1689678900);
    }

    #[test]
    fn test_stock_quote_to_wire_line() {
        let quote = StockQuote::new("SBER".to_string(), 350.50, 1000, 1689678900);
        assert_eq!(quote.to_wire_line(), "SBER|350.5|1000|1689678900\n");
    }

    #[test]
    fn test_stock_quote_from_wire_line_success() {
        let line = "SBER|350.50|1000|1689678900";
        let quote = StockQuote::from_wire_line(line).unwrap();
        assert_eq!(quote.ticker, "SBER");
        assert_eq!(quote.price, 350.50);
        assert_eq!(quote.volume, 1000);
        assert_eq!(quote.timestamp, 1689678900);
    }

    #[test]
    fn test_stock_quote_from_wire_line_invalid_format() {
        let line = "SBER|350.50";
        let result = StockQuote::from_wire_line(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_stock_quote_from_wire_line_invalid_ticker() {
        let line = "SB ER|350.50|1000|1689678900";
        let result = StockQuote::from_wire_line(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_stock_quote_from_wire_line_invalid_price() {
        let line = "SBER|abc|1000|1689678900";
        let result = StockQuote::from_wire_line(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_stock_quote_from_wire_line_invalid_volume() {
        let line = "SBER|350.50|abc|1689678900";
        let result = StockQuote::from_wire_line(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_stock_quote_from_wire_line_invalid_timestamp() {
        let line = "SBER|350.50|1000|abc";
        let result = StockQuote::from_wire_line(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_quote_stream_new() {
        // Создаем UDP сокет для привязки
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = socket.local_addr().unwrap().to_string();
        drop(socket);

        let subscribe = Subscribe::new(addr.clone(), vec!["SBER".to_string()]);
        let stream = QuoteStream::new(subscribe);
        assert!(stream.is_ok());
    }

    #[test]
    fn test_quote_stream_get_address() {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = socket.local_addr().unwrap().to_string();
        drop(socket);

        let subscribe = Subscribe::new(addr.clone(), vec!["SBER".to_string()]);
        let stream = QuoteStream::new(subscribe).unwrap();
        assert_eq!(stream.get_address(), addr);
    }

    #[test]
    fn test_quote_stream_equal_address() {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = socket.local_addr().unwrap().to_string();
        drop(socket);

        let subscribe = Subscribe::new(addr.clone(), vec!["SBER".to_string()]);
        let stream = QuoteStream::new(subscribe).unwrap();
        assert!(stream.equal_address(&addr));
        assert!(!stream.equal_address("127.0.0.1:9999"));
    }

    #[test]
    fn test_quote_stream_stop_stream() {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = socket.local_addr().unwrap().to_string();
        drop(socket);

        let subscribe = Subscribe::new(addr.clone(), vec!["SBER".to_string()]);
        let stream = QuoteStream::new(subscribe).unwrap();
        assert!(!stream.streaming.load(Ordering::Relaxed));
        
        stream.stop_stream();
        assert!(!stream.streaming.load(Ordering::Relaxed));
    }

    #[test]
    fn test_send_delivers_datagram() {
        let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
        let rx_addr = rx.local_addr().unwrap().to_string();
        rx.set_read_timeout(Some(Duration::from_secs(1))).unwrap();

        let subscribe = Subscribe::new(rx_addr.clone(), vec!["SBER".to_string()]);
        let stream = QuoteStream::new(subscribe).unwrap();
        let quote = StockQuote::new("SBER".to_string(), 100.0, 10, 1);

        stream.send(&quote, &rx_addr).unwrap();

        let mut buf = [0u8; 256];
        let n = rx.recv(&mut buf).unwrap();
        assert_eq!(&buf[..n], quote.to_wire_line().as_bytes());
    }

    #[test]
    fn test_streaming_flag_shared() {
        let subscribe = Subscribe::new("127.0.0.1:9999".to_string(), vec!["SBER".to_string()]);
        let stream = QuoteStream::new(subscribe).unwrap();
        let flag = stream.streaming_flag();

        flag.store(true, Ordering::Relaxed);
        assert!(stream.streaming.load(Ordering::Relaxed));
    }

    #[test]
    fn test_run_stream_stops_on_flag() {
        let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = rx.local_addr().unwrap().to_string();
        drop(rx);

        let subscribe = Subscribe::new(addr, vec!["SBER".to_string()]);
        let stream = Arc::new(QuoteStream::new(subscribe).unwrap());
        let s = stream.clone();

        let handle = thread::spawn(move || s.run_stream(10));
        thread::sleep(Duration::from_millis(50));
        stream.stop_stream();

        let result = handle.join().unwrap();
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_stream_starts_flag() {
        let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
        let addr = rx.local_addr().unwrap().to_string();
        drop(rx);

        let subscribe = Subscribe::new(addr, vec!["SBER".to_string()]);
        let stream = Arc::new(QuoteStream::new(subscribe).unwrap());
        assert!(!stream.streaming.load(Ordering::Relaxed));

        let s = stream.clone();
        let handle = thread::spawn(move || s.run_stream(10));
        thread::sleep(Duration::from_millis(30));
        assert!(stream.streaming.load(Ordering::Relaxed));

        stream.stop_stream();
        let _ = handle.join();
    }

    #[test]
    fn test_send_to_wrong_address_does_not_panic() {
        let subscribe = Subscribe::new("127.0.0.1:9998".to_string(), vec!["SBER".to_string()]);
        let stream = QuoteStream::new(subscribe).unwrap();
        let quote = StockQuote::new("SBER".to_string(), 100.0, 10, 1);

        // Отправка на несуществующий адрес — UDP не гарантирует доставку, но не паникует
        let result = stream.send(&quote, "127.0.0.1:19999");
        assert!(result.is_ok());
    }

    #[test]
    fn test_quote_stream_multiple_stops_idempotent() {
        let subscribe = Subscribe::new("127.0.0.1:9997".to_string(), vec!["SBER".to_string()]);
        let stream = QuoteStream::new(subscribe).unwrap();

        stream.stop_stream();
        stream.stop_stream();
        stream.stop_stream();
        assert!(!stream.streaming.load(Ordering::Relaxed));
    }

    #[cfg(feature = "random")]
    #[test]
    fn test_quote_stream_random() {
        let tickers = vec!["SBER".to_string(), "GAZP".to_string()];
        let quote = QuoteStream::random(&tickers);
        assert!(quote.ticker == "SBER" || quote.ticker == "GAZP");
        assert!(quote.price >= 100.0 && quote.price <= 5000.0);
        assert!(quote.volume >= 10 && quote.volume <= 1000);
    }

    #[test]
    fn test_stock_quote_clone() {
        let quote1 = StockQuote::new("SBER".to_string(), 350.50, 1000, 1689678900);
        let quote2 = quote1.clone();
        assert_eq!(quote1, quote2);
    }

    #[test]
    fn test_stock_quote_debug() {
        let quote = StockQuote::new("SBER".to_string(), 350.50, 1000, 1689678900);
        let debug_str = format!("{:?}", quote);
        assert!(debug_str.contains("SBER"));
    }
}
