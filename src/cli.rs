// Command line interface
use clap_derive::Parser;
use std::path::PathBuf;

#[derive(Parser, Clone, Debug)]
pub struct Args {
    #[arg(short, long, default_value = "client.toml")]
    pub config: PathBuf,
    #[arg(long)]
    // pub ticket: Option<ServiceTicket>,
    #[arg(long)]
    pub count: Option<i32>,
}
