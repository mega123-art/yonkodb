mod config;
mod server;
use clap::Parser;
use config::Config;
fn main() {
    let config = Config::parse();

    println!("spinning up yonko db 🏴‍☠️");
    server::run_sync_tcp_server(&config);
}
