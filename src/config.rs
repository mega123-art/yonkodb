use clap::Parser;
#[derive(Parser, Debug)]
#[command(author, version, about = "currently arpit bhayani's redis playlist impl but in rust", long_about = None)]
pub struct Config {
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    pub host: String,
    #[arg(short, long, default_value = "7379")]
    pub port: u16,
}
