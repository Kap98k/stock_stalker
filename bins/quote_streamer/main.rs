//! Udp сервер котировок TCP/UDP/PING
use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::collections::HashMap;
use clap::{Parser, ArgGroup};
use stock_stalker::error::CommandError;
use stock_stalker::quote::QuoteStream;
use stock_stalker::{protocol::*, GEN_QUOTE_RANGE_MAX};

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

struct Subscribers{
    subscriber_host: String,
    streams: HashMap<String, Arc<Mutex<QuoteStream>>>
}

impl Subscribers {
    pub fn add(&mut self, quote_stream: Arc<Mutex<QuoteStream>>){
        let mut a = String::new();
        {
            let mut _qs = quote_stream.lock().unwrap();
            a = _qs.get_address();
        }
        self.streams.insert(a, quote_stream);
    }

    pub fn remove(&mut self, addr: &str){
        if let Some(qs) = self.streams.get_mut(addr){
            {
                let mut _qs = qs.lock().unwrap();
                _qs.stop_stream();
            }
            self.streams.remove(addr);
        }
    }

    pub fn find_by_addr(&self, addr: &str) -> bool{
        return self.streams.contains_key(addr)
    }
}

fn handle_connection(stream: TcpStream, subscribers: Arc<Mutex<Vec<Subscribers>>>) {
    let mut writer = stream.try_clone().expect("failed to clone stream");
    let mut reader = BufReader::new(stream);
    
    let mut line = String ::new(); 
    loop{
        line.clear();
        match reader.read_line(&mut line){
            Ok(0) => {return}
            Ok(_)=>{
                println!("New command {}", line);
                if line.contains(STOP){
                    let command = line.trim_start();
                    let mut answer = String::new();
                    match check_stop_command(command){
                        Ok(addr) => {
                            let mut s_list = subscribers.lock().unwrap();
                            let host = writer.peer_addr().unwrap();
                            if let Some(result) = s_list.iter_mut().find(|s| s.find_by_addr(&host.to_string())){
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
                            //#ISSUE дописать создание вектора подписчиков и запуск UDP сокета для отправки котировок
                            let mut s_list = subscribers.lock().unwrap();
                            let qs = QuoteStream::new(s);
                            match qs {
                                Ok(qs) =>{
                                    let host = writer.peer_addr().unwrap();
                                    if let Some(result) = s_list.iter_mut().find(|s| s.find_by_addr(&host.to_string())){
                                        let quote_stream = Arc::new(Mutex::new(qs));
                                        result.add(quote_stream.clone());
                                        let handle = thread::spawn(move || {
                                            let mut qs = quote_stream.lock().unwrap();
                                            println!("Запускаем передачу котировок для {}", qs.get_address());
                                            let _ = qs.run_stream(GEN_QUOTE_RANGE_MAX as u64);
                                        });
                                        answer = "Ok\n".to_string();
                                    }
                                    else{
                                        let mut hs = HashMap::new();
                                        hs.insert(qs.get_address(), Arc::new(Mutex::new(qs)));
                                        s_list.push(Subscribers{subscriber_host:host.to_string(), streams:hs});
                                    }
                                }
                                Err(e)=>{
                                    println!("Ошибка при создании стриминга: {}", e);
                                    //answer = CommandError::InvalidAddress(e.to_string()).to_string();
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
            }
            Err(_) => { return }
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