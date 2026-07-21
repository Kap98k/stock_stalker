//! Udp сервер котировок TCP/UDP/PING
use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use std::thread;
use std::collections::HashMap;
use clap::{ArgGroup, Parser};

use stock_stalker::error::CommandError;
use stock_stalker::quote::QuoteStream;
use stock_stalker::{GEN_QUOTE_RANGE_MAX, PING_TIMEOUT_SECONDS, protocol::*};

#[derive(Parser)]
#[command(
    name = "Quote Stream Server",
    author = "Kap98k",
    version = "0.33",
    about = "Шлет котировки или не только их?",
    long_about = None,
    group(ArgGroup::new("data").args(["tickers", "file_path_tickers"]).required(true))
)]

struct Cli{
    #[arg(short, long)]
    address: String,

    #[arg(short, long, required = true)]
    port: u16,

    #[arg(short)]
    tickers : Vec<String>,

    #[arg(short)]
    file_path_tickers:String
}

/// Запись о подписчике: сам поток (под Mutex для Send+Sync) и флаг остановки.
struct Subscribers{
    pub subscriber_host: String,
    /// Ключ: UDP-адрес → (поток, флаг-остановки)
    streams: HashMap<String, (Arc<Mutex<QuoteStream>>, Arc<AtomicBool>)>,
}

impl Subscribers {
    /// Добавляет стрим: сохраняет и Arc<Mutex<QuoteStream>>, и флаг для остановки.
    pub fn add(&mut self, qs: Arc<Mutex<QuoteStream>>){
        let (addr, flag) = {
            let _qs = qs.lock().unwrap();
            (_qs.get_address(), _qs.streaming_flag())
        };
        self.streams.insert(addr, (qs, flag));
    }

    /// Останавливает стрим по UDP-адресу.
    pub fn remove(&mut self, addr: &str){
        if let Some((_qs, flag)) = self.streams.get(addr){
            flag.store(false, Ordering::Relaxed);
        }
        self.streams.remove(addr);
    }

    /// Останавливает все стримы этого подписчика.
    pub fn remove_all(&mut self){
        for (_key, (_qs, flag)) in self.streams.iter(){
            flag.store(false, Ordering::Relaxed);
        }
        self.streams.clear();
    }

}

fn handle_connection(stream: TcpStream, subscribers: Arc<Mutex<Vec<Subscribers>>>) {
    let mut writer = stream.try_clone().expect("Ошибка при клонировании потока");
    stream.set_read_timeout(Some(Duration::from_secs(1))).expect("set_read_timeout failed");
    let mut reader = BufReader::new(stream);

    let mut line = String ::new();
    let mut last_ping = Instant::now();
    loop{
        line.clear();
        match reader.read_line(&mut line){
            Ok(0) => {
                // Клиент отключился — чистим все его стримы
                let host = writer.peer_addr().unwrap();
                let mut s_list = subscribers.lock().unwrap();
                if let Some(result) = s_list.iter_mut().find(|s| s.subscriber_host == host.to_string()){
                    result.remove_all();
                }
                return;
            }
            Ok(_)=>{
                println!("New command {}", line);
                if line.contains(STOP){
                    let command = line.trim_start();
                    let mut answer = String::new();
                    match check_stop_command(command){
                        Ok(addr) => {
                            let mut s_list = subscribers.lock().unwrap();
                            let host = writer.peer_addr().unwrap();
                            if let Some(result) = s_list.iter_mut().find(|s| s.subscriber_host == host.to_string()){
                                result.remove(&addr.trim());
                                answer = "Ok\n".to_string();
                            }
                            else{
                                answer = CommandError::AddressNotFound(addr).to_string()
                            }
                        }
                        Err(e) =>{
                            answer = e.to_string();
                        }
                    }
                    let _ = writer.write_all(answer.as_bytes());
                    let _ = writer.flush();
                }

                if line.contains(STREAM){
                    let command = line.trim_start();
                    let mut answer = String::new();
                    match check_stream_command(command){
                        Ok(s) => {
                            let mut s_list = subscribers.lock().unwrap();
                            let qs = QuoteStream::new(s);
                            match qs {
                                Ok(qs) =>{
                                    let qs = Arc::new(Mutex::new(qs));
                                    let host = writer.peer_addr().unwrap();
                                    if let Some(result) = s_list.iter_mut().find(|s| s.subscriber_host == host.to_string()){
                                        result.add(qs.clone());
                                        let _handle = thread::spawn(move || {
                                            let qs = qs.lock().unwrap();
                                            println!("Запускаем передачу котировок для {}", qs.get_address());
                                            let _ = qs.run_stream(GEN_QUOTE_RANGE_MAX as u64);
                                        });
                                    }
                                    else{
                                        let udp_address = {
                                            let _qs = qs.lock().unwrap();
                                            _qs.get_address()
                                        };
                                        let flag = {
                                            let _qs = qs.lock().unwrap();
                                            _qs.streaming_flag()
                                        };
                                        let mut hs = HashMap::new();
                                        hs.insert(udp_address, (qs.clone(), flag));
                                        s_list.push(Subscribers{subscriber_host:host.to_string(), streams:hs});
                                        let _handle = thread::spawn(move || {
                                            let qs = qs.lock().unwrap();
                                            println!("Запускаем передачу котировок для {}", qs.get_address());
                                            let _ = qs.run_stream(GEN_QUOTE_RANGE_MAX as u64);
                                        });
                                    }
                                    answer = "Ok\n".to_string();
                                }
                                Err(e)=>{
                                    println!("Ошибка при создании стриминга: {}", e);
                                }
                            }
                        }
                        Err(e) =>{
                            answer = e.to_string();
                        }
                    }
                    let _ = writer.write_all(answer.as_bytes());
                    let _ = writer.flush();
                }
                if line.contains(PING){
                    let command = line.trim();
                    if command.starts_with(PING){
                        println!("Получен PING от клиента {}", &writer.peer_addr().unwrap().to_string());
                        last_ping = Instant::now();
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                // read timeout — проверим PING ниже
            }
            Err(_) => { return }
        }
        if last_ping.elapsed() >= Duration::from_secs(PING_TIMEOUT_SECONDS as u64) {
            let mut s_list = subscribers.lock().unwrap();
            let host = writer.peer_addr().unwrap();
            if let Some(result) = s_list.iter_mut().find(|s| s.subscriber_host == host.to_string()){
                result.remove_all();
            }
            break;
        }
    }

}

fn main() -> std::io::Result<()>{
    let cli = Cli::parse();
    let tcp_server = TcpListener::bind(format!("{}:{}", cli.address, cli.port))?;
    println!("Server listening on port {}.", cli.port);
    let subscribers_list = Arc::new(Mutex::new(Vec::new()));
    //#ISUUE если в параметрах передается файл, а не массив котировок.
    // То надо организовывать доступ к нему
    for stream in tcp_server.incoming(){
        match stream{
            Ok(stream) => {
                let sl = Arc::clone(&subscribers_list);
                thread::spawn(move || handle_connection(stream, sl));
            }
            Err(e) => {
                eprintln!("Connection failed. Error : {} ",e);
            }
        }
    }

    Ok(())
}