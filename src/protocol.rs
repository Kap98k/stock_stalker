use std::net::IpAddr;
use std::str::FromStr;

use crate::{error::{CommandError}, tickers::check_tickers};

pub const STREAM_COMMAND:&str = "STREAM {} {}\n";
pub const STOP_COMMAND: &str = "STOP {}\n";

pub const STREAM: &str = "STREAM";
pub const PING: &str = "PING";
pub const PONG: &str ="PONG";
pub const STOP: &str = "STOP";

pub struct Subscribe{
    pub address: String,
    pub tickers: Vec<String>
}

impl Subscribe {
    pub fn new(address:String, tickers :Vec<String>) -> Self{
        Self{address, tickers}
    }


}

pub fn check_stop_command(command:&str) -> Result<String, CommandError>{
    if command.starts_with(STOP){
        let command_parts: Vec<&str> = command.splitn(2, " ").collect();
        let address = command_parts[1];
        match check_address(address){
            Err(e) => return Err(e),
            Ok(_) => Ok(address.to_string())
        }
    }
    else{
        Err(CommandError::InvalidCommand(command.to_string()))
    }
}

pub fn check_stream_command(command: &str) -> Result<Subscribe, CommandError>{
    if command.starts_with(STREAM){
        let command_parts: Vec<&str> = command.splitn(3, " ").collect();
        let address = command_parts[1];
        match check_address(address){
            Err(e) => return Err(e),
            _=>{
                let req_tickers : Vec<&str> = command_parts[2].trim().split(", ").collect();
                if req_tickers.is_empty() {
                    return Err(CommandError::EmptyTickerList)
                }

                match check_tickers(req_tickers){
                    Err(e) => return Err(e),
                    Ok(tickers) => {
                        Ok(Subscribe::new(address.to_string(), tickers))
                    }
                }
            }
        }
    }
    else{
        Err(CommandError::InvalidCommand(command.to_string()))
    }
}

fn check_address(input: &str)-> Result<(), CommandError>{
    if let Some(pos) = input.rfind(':') {
        let (host_part, port_part) = input.split_at(pos);
        
        let host = &host_part[..pos];
        let port_str = &port_part[1..]; 

        // 1. Валидируем порт
        if port_str.parse::<u16>().is_ok(){
            match IpAddr::from_str(host) {
                Ok(_) => return Ok(()),
                Err(_) => return Err(CommandError::InvalidAddress(input.to_string()))
            }
        }
    }
    Err(CommandError::InvalidAddress(input.to_string()))
}