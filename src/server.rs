use crate::config::Config;
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
fn handle_client(stream: &mut TcpStream, con_clients: &mut i32, peer_addr: SocketAddr) {
    let mut buf = [0; 512];

    loop {
        match stream.read(&mut buf) {
            Ok(0) => {
                *con_clients -= 1;
                println!(
                    "client dissconnected {}, concurrent clients {}",
                    peer_addr, con_clients
                );
                break;
            }
            Ok(n) => {
                let cmd = String::from_utf8_lossy(&buf[..n]);
                println!("command: {}", cmd.trim_end());

                if let Err(e) = stream.write_all(&buf[..n]) {
                    eprintln!("err write: {}", e);
                    break;
                }
            }
            Err(e) => {
                *con_clients -= 1;
                eprintln!(
                    "client dissconnected with error: {}, concurrent clients {}",
                    e, con_clients
                );
                break;
            }
        }
    }
}
