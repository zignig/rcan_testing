// Command line interface
use clap_derive::Parser;
use iroh::PublicKey;
use std::path::PathBuf;

#[derive(Parser, Clone, Debug)]
pub struct Args {
    #[arg(short, long, default_value = "client.toml")]
    pub config: PathBuf,
    #[clap(subcommand)]
    pub command: Option<Command>,
    #[arg(long)]
    // pub ticket: Option<ServiceTicket>,
    #[arg(long)]
    pub count: Option<i32>,
}

#[derive(Parser, Clone, Debug)]
pub enum Command {
    Issue {
        key: PublicKey,
        #[arg(long,short, default_value_t = false)]
        status: bool,
        #[arg(long,short, default_value_t = false)]
        all: bool,
        #[arg(long,short, default_value = "2d")]
        duration: Option<String>
    }
}