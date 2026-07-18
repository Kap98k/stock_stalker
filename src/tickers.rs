use std::collections::HashSet;
use once_cell::sync::Lazy;
use std::fs;
use std::sync::RwLock;
use crate::error::*;

pub static TICKERS_LIST: Lazy<RwLock<HashSet<String>>> = Lazy::new(||{
    let content = fs::read_to_string("tickers.txt");
    match content{
        Ok(data) => {
            let items: HashSet<String> = data
                .lines()
                .map(|s| s.trim().to_string())
                .collect();
                
            RwLock::new(items)
        }
        Err(e) =>{
            println!("Ошибка при обработке файла тикеров: {}", e);
            RwLock::new(HashSet::new())
        }
    }
});

pub fn check_tickers(req_tickers: Vec<&str>) -> Result<Vec<String>, CommandError> {
    let mut tickers: Vec<String> = Vec::new();
    let ticker_list = TICKERS_LIST.read().unwrap();
    // Или стоит отправлять те тикеры, что нашлись?
    for ticker in req_tickers.into_iter(){
        if ticker_list.contains(&ticker.to_string()) == false {
            return Err(CommandError::TickerNotFound(ticker.to_string()))
        }
        tickers.push(ticker.trim().to_string());
    }
    Ok(tickers)
}