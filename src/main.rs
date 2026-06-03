mod aync_server;
mod config;
mod core;
use clap::Parser;
use config::Config;
fn main() {
    let config = Config::parse();

    println!("spinning up yonko db 🏴‍☠️");
    aync_server::run_async_tcp_server(&config);
}
