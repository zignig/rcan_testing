use clap::Parser;
/// Rcan testing
///
///
use n0_error::Result;
use tracing::info;

use rcan_testing::{Args, IdentityApi, Settings};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Arguments
    let args = Args::parse();
    info!("cmd line {:?}", args);

    // Settings
    let config = Settings::load(args.config)?;
    info!("{:#?}", config);

    // Create the identity client 
    let id_service = IdentityApi::new();
    let id = id_service.client();
    println!("{:#?}",id.list().await);
    Ok(())
}
