//! Клиент получающий данные с сервера котировок
use clap::Parser;
use stock_stalker::protocol::STOP;
use stock_stalker::quote::StockQuote;
use stock_stalker::PING_INTERVAL_SECONDS;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "subscriber")]
#[command(about = "Клиент для получения котировок")]
struct Args {
    /// TCP адрес сервера котировок (например, 127.0.0.1:8080)
    #[arg(short, long)]
    tcp_server: String,

    /// UDP адрес для получения котировок (например, 127.0.0.1:9000)
    #[arg(short, long)]
    udp_address: String,

    /// Список тикеров через запятую с пробелом (например, SBER, GAZP, LKOH)
    #[arg(short = 'k', long)]
    tickers: String,
}

fn main() {
    let args = Args::parse();

    println!("Подключение к серверу {}", args.tcp_server);

    // Подключаемся к TCP серверу
    let mut tcp_stream = match connect_to_server(&args.tcp_server) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Ошибка подключения к серверу: {}", e);
            return;
        }
    };

    // Отправляем команду STREAM
    let stream_command = format!("STREAM {} {}\n", args.udp_address, args.tickers);
    if let Err(e) = send_command(&mut tcp_stream, &stream_command) {
        eprintln!("Ошибка отправки команды STREAM: {}", e);
        return;
    }

    // Читаем ответ от сервера
    let response = read_response(&mut tcp_stream);
    println!("Ответ от сервера: {}", response);

    if response.starts_with("Ok") || response.contains("Ok") {
        println!("Подписка успешно оформлена");
        println!("Ожидание котировок на UDP адресе {}...", args.udp_address);

        // Запускаем приём котировок в отдельном потоке
        let udp_address = args.udp_address.clone();
        let ticker_list = args.tickers.clone();
        thread::spawn(move || {
            receive_quotes(&udp_address, &ticker_list);
        });

        // Запускаем фоновую отправку PING каждые PING_INTERVAL_SECONDS
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        let mut tcp_ping = tcp_stream.try_clone().expect("failed to clone stream");
        let ping_handle = thread::spawn(move || {
            while r.load(Ordering::Relaxed) {
                let _ = send_command(&mut tcp_ping, "PING\n");
                thread::sleep(Duration::from_secs(PING_INTERVAL_SECONDS as u64));
            }
        });

        // Основной поток обрабатывает STOP и exit
        handle_commands(tcp_stream, running, &args.udp_address);
        let _ = ping_handle.join();
    } else {
        eprintln!("Ошибка при оформлении подписки: {}", response);
    }
}

fn connect_to_server(server: &str) -> Result<TcpStream, std::io::Error> {
    let max_retries = 5;
    let mut attempt = 0;

    while attempt < max_retries {
        match TcpStream::connect(server) {
            Ok(stream) => return Ok(stream),
            Err(e) => {
                attempt += 1;
                eprintln!("Попытка подключения {} failed: {}", attempt, e);
                if attempt < max_retries {
                    thread::sleep(Duration::from_secs(2));
                } else {
                    return Err(e);
                }
            }
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "Max retries exceeded",
    ))
}

fn send_command(stream: &mut TcpStream, command: &str) -> Result<(), std::io::Error> {
    stream.write_all(command.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn read_response(stream: &mut TcpStream) -> String {
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    let _ = reader.read_line(&mut response);
    response.trim().to_string()
}

fn receive_quotes(udp_address: &str, ticker_list: &str) {
    println!("Начинаем приём котировок...");

    // Разбираем список тикеров для отладки
    let tickers: Vec<&str> = ticker_list.split(',').map(|s| s.trim()).collect();
    println!("Подписаны на тикеры: {:?}", tickers);

    // Привязываемся к UDP сокету
    match UdpSocket::bind(udp_address) {
        Ok(socket) => {
            let mut buf = [0u8; 4096];
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((len, _addr)) => {
                        let line = String::from_utf8_lossy(&buf[..len]);
                        if let Ok(quote) = StockQuote::from_wire_line(line.trim()) {
                            println!(
                                "[{}] Цена: {} | Объём: {} | Время: {}",
                                quote.ticker, quote.price, quote.volume, quote.timestamp
                            );
                        } else {
                            println!("Не удалось распарсить котировку: {}", line);
                        }
                    }
                    Err(e) => {
                        eprintln!("Ошибка приёма UDP данных: {}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Ошибка привязки UDP сокета: {}", e);
        }
    }
}

fn handle_commands(mut tcp_stream: TcpStream, running: Arc<AtomicBool>, udp_address: &str) {
    println!("\nДоступные команды:");
    println!("  stop    - остановить поток котировок и выйти");
    println!("  exit    - выйти из клиента");
    println!();

    let mut input = String::new();
    loop {
        input.clear();
        print!("> ");
        std::io::stdout().flush().unwrap();

        match std::io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {
                let command = input.trim();
                match command {
                    "stop" => {
                        running.store(false, Ordering::Relaxed);
                        let stop_cmd = format!("STOP {}\n", udp_address);
                        if let Err(e) = send_command(&mut tcp_stream, &stop_cmd) {
                            eprintln!("Ошибка отправки STOP: {}", e);
                        }
                        let response = read_response(&mut tcp_stream);
                        println!("STOP response: {}", response);
                        println!("Выход...");
                        break;
                    }
                    "exit" => {
                        running.store(false, Ordering::Relaxed);
                        println!("Выход...");
                        break;
                    }
                    _ => {
                        println!("Неизвестная команда. Используйте: stop, exit");
                    }
                }
            }
            Err(e) => {
                eprintln!("Ошибка чтения команды: {}", e);
                break;
            }
        }
    }
}