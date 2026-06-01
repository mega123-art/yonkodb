use crate::core::cmd::YonkoCmd;
use crate::core::resp::RespType;
use std::io::Write;
use std::net::TcpStream;

pub fn eval_and_respond(cmd: &YonkoCmd, stream: &mut TcpStream) -> Result<(), String> {
    println!("command: {}", cmd.cmd);

    match cmd.cmd.as_str() {
        "PING" => eval_ping(&cmd.args, stream),
        _ => {
            // Default to PING for now if unknown
            eval_ping(&cmd.args, stream)
        }
    }
}

//ping
fn eval_ping(args: &[String], stream: &mut TcpStream) -> Result<(), String> {
    if args.len() >= 2 {
        return Err("ERR wrong number of arguments for 'ping' command".to_string());
    }

    let response = if args.is_empty() {
        RespType::encode_string("PONG", true)
    } else {
        RespType::encode_string(&args[0], false)
    };

    stream.write_all(&response).map_err(|e| e.to_string())
}
