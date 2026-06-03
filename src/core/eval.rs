use crate::core::cmd::YonkoCmd;
use crate::core::resp::RespType;
use std::io::Write;

pub fn eval_and_respond(cmd: &YonkoCmd, stream: &mut impl Write) -> Result<(), String> {
    match cmd.cmd.as_str() {
        "PING" => eval_ping(&cmd.args, stream),
        _ => {
            eval_ping(&cmd.args, stream)
        }
    }
}

fn eval_ping(args: &[String], stream: &mut impl Write) -> Result<(), String> {
    if args.len() >= 2 {
        return Err("ERR wrong number of arguments for 'ping' command".to_string());
    }

    if args.is_empty() {
        stream.write_all(b"+PONG\r\n").map_err(|e| e.to_string())
    } else {
        let response = RespType::encode_string(&args[0], false);
        stream.write_all(&response).map_err(|e| e.to_string())
    }
}
