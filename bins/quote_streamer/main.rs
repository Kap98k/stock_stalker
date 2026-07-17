//! Udp сервер котировок TCP/UDP/PING
use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader, Write};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use clap::{Parser, Arg, ArgGroup};
use stock_stalker::tickers::TickerStream;
use stock_stalker::protocol::*;

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
    streams: Vec<TickerStream>
}

fn handle_connection(stream: TcpStream) {
    let mut writer = stream.try_clone().expect("failed to clone stream");
    let mut reader = BufReader::new(stream);
    
    let mut line = String ::new(); 
    loop{
        line.clear();
        match reader.read_line(&mut line){
            Ok(0) => {return}
            Ok(_)=>{
                println!("New command {}", line);
                let input = line.trim();
                if input.eq_ignore_ascii_case(STOP){
                    
                }

                if input.contains(STREAM){
                    let command = input.trim_start();
                    match check_command(command){
                        Ok(s) => {
                            //#ISSUE дописать создание вектора подписчиков и запуск UDP сокета для отправки котировок
                        }
                        Err(e) =>{
                            let _ = writer.write_all(e.to_string().as_bytes());
                            let _ = writer.flush();
                        }
                    }

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
    //#ISUUE если в параметрах передается файл, а не массив котировок.
    // То надо организовывать доступ к нему
    for stream in tcp_server.incoming(){
        match stream{
            Ok(stream) => {
                thread::spawn(move || handle_connection(stream));
            }
            Err(e) => {
                eprintln!("Connection failed. Error : {} ",e);
            }
        }
    }

    Ok(())
}