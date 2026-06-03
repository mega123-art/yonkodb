
use crate::config::Config;
use crate::core::cmd::YonkoCmd;
use crate::core::eval::eval_and_respond;
use crate::core::resp::decode;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{Read, Write};

const SERVER: Token = Token(0);

pub fn run_async_tcp_server(config: &Config) {
    let address = format!("{}:{}", config.host, config.port);
    println!("starting async server on {}", address);
    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(2000);
    let addr = address.parse().unwrap();
    let mut listener = TcpListener::bind(addr).unwrap();

    poll.registry().register(&mut listener, SERVER, Interest::READABLE).unwrap();

    let mut clients: HashMap<Token, TcpStream> = HashMap::new();
    let mut next_token: usize = 1;
    let mut connected_clients: usize = 0;

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    loop {
                        match listener.accept() {
                            Ok((mut stream, addr)) => {
                                connected_clients += 1;
                                println!("client connected {} concurrent clients {}", addr, connected_clients);
                                let token = Token(next_token);
                                next_token = next_token.wrapping_add(1);
                                poll.registry().register(&mut stream, token, Interest::READABLE).unwrap();
                                clients.insert(token, stream);
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                break;
                            }
                            Err(e) => {
                                eprintln!("failed to accept connection {}", e);
                                break;
                            }
                        }
                    }
                }
                token => {
                    let mut should_disconnect = false;
                    if let Some(client_stream) = clients.get_mut(&token) {
                        match read_command(client_stream) {
                            Ok(cmd) => {
                                if cmd.cmd == "PING" && cmd.args.is_empty() {
                                    if let Err(_) = client_stream.write_all(b"+PONG\r\n") {
                                        should_disconnect = true;
                                    }
                                } else {
                                    if let Err(e) = eval_and_respond(&cmd, client_stream) {
                                        let err_msg = format!("-ERR {}\r\n", e);
                                        let _ = client_stream.write_all(err_msg.as_bytes());
                                    }
                                }
                            }
                            Err(e) => {
                                if e != "WouldBlock" {
                                    should_disconnect = true;
                                    if e != "EOF" {
                                        eprintln!("client error: {}", e);
                                        let err_msg = format!("-ERR {}\r\n", e);
                                        let _ = client_stream.write_all(err_msg.as_bytes());
                                    }
                                }
                            }
                        }
                    }
                    if should_disconnect {
                        connected_clients -= 1;
                        clients.remove(&token);
                        println!("client disconnected, concurrent clients: {}", connected_clients);
                    }
                }
            }
        }
    }
}

fn read_command(c: &mut TcpStream) -> Result<YonkoCmd, String> {
    let mut buf = [0; 512];
    match c.read(&mut buf) {
        Ok(0) => return Err("EOF".to_string()),
        Ok(n) => {
            if n == 14 && (&buf[..14] == b"*1\r\n$4\r\nPING\r\n" || &buf[..14] == b"*1\r\n$4\r\nping\r\n") {
                return Ok(YonkoCmd {
                    cmd: "PING".to_string(),
                    args: Vec::new(),
                });
            }
            if n == 6 && (&buf[..6] == b"PING\r\n" || &buf[..6] == b"ping\r\n") {
                return Ok(YonkoCmd {
                    cmd: "PING".to_string(),
                    args: Vec::new(),
                });
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
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
            return Err("WouldBlock".to_string());
        }
        Err(e) => return Err(e.to_string()),
    }
}
