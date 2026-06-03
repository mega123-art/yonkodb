use crate::config::Config;
use crate::core::cmd::YonkoCmd;
use crate::core::eval::eval_and_respond;
use crate::core::resp::decode;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
pub fn run_sync_tcp_server(config: &Config) {
    let address = format!("{}:{}", config.host, config.port);
    println!("starting a synchronous TCP server on {}", address);

    let listener = TcpListener::bind(&address).expect("Failed to bind to address");
    let mut con_clients = 0;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                con_clients = con_clients + 1;
                let peer_addr = stream.peer_addr().unwrap();
                println!(
                    "client connected with address: {}, concurrent clients {}",
                    peer_addr, con_clients
                );
                handle_client(&mut stream, &mut con_clients, peer_addr);
            }
            Err(e) => {
                eprintln!("failed to accept connection: {}", e);
            }
        }
    }
}
fn read_command(c: &mut TcpStream) -> Result<YonkoCmd, String> {
    let mut buf = [0; 512];
    let n = c.read(&mut buf).map_err(|e| e.to_string())?;

    if n == 0 {
        return Err("EOF".to_string());
    }

    let resp = decode(&buf[..n])?;

    let tokens = resp.to_string_vec()?;

    if tokens.is_empty() {
        return Err("Empty command".to_string());
    }

    Ok(YonkoCmd {
        cmd: tokens[0].to_ascii_uppercase(),
        args: tokens[1..].to_vec(),
    })
}

fn handle_client(stream: &mut TcpStream, con_clients: &mut i32, peer_addr: SocketAddr) {
    loop {
        match read_command(stream) {
            Ok(cmd) => {
                if let Err(e) = eval_and_respond(&cmd, stream) {
                    let err_msg = format!("-ERR {}\r\n", e);
                    let _ = stream.write_all(err_msg.as_bytes());
                }
            }
            Err(e) => {
                *con_clients -= 1;

                if e == "EOF" {
                    println!(
                        "client disconnected {}, concurrent clients {}",
                        peer_addr, con_clients
                    );
                } else {
                    eprintln!(
                        "client disconnected with error: {}, concurrent clients {}",
                        e, con_clients
                    );
                    let err_msg = format!("-ERR {}\r\n", e);
                    let _ = stream.write_all(err_msg.as_bytes());
                }
                break;
            }
        }
    }
}

