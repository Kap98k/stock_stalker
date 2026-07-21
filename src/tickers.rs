//! Управление списком доступных тикеров.
//!
//! ## Описание
//!
//! Модуль предоставляет функциональность для загрузки и проверки тикеров из файла.
//! Использует ленивую инициализацию для загрузки списка тикеров при первом доступе.

use std::collections::HashSet;
use once_cell::sync::Lazy;
use std::fs;
use std::sync::RwLock;
use crate::error::*;

/// Глобальный список доступных тикеров.
///
/// ## Описание
///
/// Статическая переменная, содержащая множество тикеров, загруженных из файла `tickers.txt`.
/// Загружается при первом обращении (ленивая инициализация).
///
/// ## Формат файла
///
/// Файл должен содержать один тикер на строку. Пустые строки и пробелы в начале/конце игнорируются.
///
/// ## Пример
///
/// ```text
/// SBER
/// GAZP
/// LKOH
/// ROSN
/// ```
pub static TICKERS_LIST: Lazy<RwLock<HashSet<String>>> = Lazy::new(|| {
    let content = fs::read_to_string("tickers.txt");
    match content {
        Ok(data) => {
            let items: HashSet<String> = data
                .lines()
                .map(|s| s.trim().to_string())
                .collect();
            
            RwLock::new(items)
        }
        Err(e) => {
            println!("Ошибка при обработке файла тикеров: {}", e);
            RwLock::new(HashSet::new())
        }
    }
});

/// Проверяет, что все указанные тикеры существуют в списке доступных.
///
/// ## Параметры
///
/// - `req_tickers` - список тикеров для проверки
///
/// ## Возвращает
///
/// - `Ok(Vec<String>)` - вектор с тикерами, если все тикеры найдены
/// - `Err(CommandError)` - ошибка, если хотя бы один тикер не найден

pub fn check_tickers(req_tickers: Vec<&str>) -> Result<Vec<String>, CommandError> {
    let mut tickers: Vec<String> = Vec::new();
    let ticker_list = TICKERS_LIST.read().unwrap();
    // Или стоит отправлять те тикеры, что нашлись?
    for ticker in req_tickers.into_iter() {
        if !ticker_list.contains(&ticker.to_string()) {
            return Err(CommandError::TickerNotFound(ticker.to_string()))
        }
        tickers.push(ticker.trim().to_string());
    }
    Ok(tickers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Глобальный мьютекс для сериализации тестов, работающих с TICKERS_LIST.
    static TICKERS_TEST_MUTEX: Mutex<()> = Mutex::new(());

    /// Перезаписывает глобальный TICKERS_LIST на время теста.
    fn with_tickers<F: FnOnce()>(tickers: &[&str], test: F) {
        let _guard = TICKERS_TEST_MUTEX.lock().unwrap();

        let old: Vec<String> = TICKERS_LIST.read().unwrap().iter().cloned().collect();
        {
            let mut list = TICKERS_LIST.write().unwrap();
            list.clear();
            for t in tickers {
                list.insert(t.to_string());
            }
        }

        test();

        {
            let mut list = TICKERS_LIST.write().unwrap();
            list.clear();
            for t in old {
                list.insert(t);
            }
        }
    }

    #[test]
    fn test_check_tickers_all_found() {
        with_tickers(&["SBER", "GAZP", "LKOH"], || {
            let result = check_tickers(vec!["SBER", "GAZP"]);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), vec!["SBER", "GAZP"]);
        });
    }

    #[test]
    fn test_check_tickers_not_found() {
        with_tickers(&["SBER"], || {
            let result = check_tickers(vec!["MISSING"]);
            assert!(result.is_err());
            match result {
                Err(CommandError::TickerNotFound(t)) => assert_eq!(t, "MISSING"),
                _ => panic!("expected TickerNotFound"),
            }
        });
    }

    #[test]
    fn test_check_tickers_empty_input() {
        with_tickers(&["SBER"], || {
            let result = check_tickers(vec![]);
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        });
    }

    #[test]
    fn test_check_tickers_trims_whitespace() {
        with_tickers(&["SBER"], || {
            let result = check_tickers(vec!["  SBER  "]);
            assert!(result.is_err());
        });
    }
}
