use clap::Parser;
use iroh::Endpoint;
use iroh::endpoint::presets;
use iroh::protocol::RouterBuilder;
/// Rcan testing
///
///
use n0_error::Result;
use rcan_testing::cli::Command;
use rcan_testing::connect::AuthClient;
use tracing::error;
use tracing::info;

use rcan_testing::auth;
use rcan_testing::incoming;
use rcan_testing::{Args, IdentityApi, Settings};
use rcan_testing::caps;
use rcan_testing::irpc;


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Arguments
    let args = Args::parse();
    // info!("cmd line {:#?}", args);

    // Settings
    let config = Settings::load(args.config)?;
    // info!("{:#?}", config);

    println!("id {}", config.public());

    // Create the identity client
    let id_service = IdentityApi::new();
    let id_client = id_service.client();

    
    if let Some(command) = args.command  { 
        match command { 
            Command::Issue{key,status,all,duration} => { 
                let _cap = caps::issue(key,status,all,duration,config.secret());
            }
        }
    }

    // the rcan editor 
    let rc_edit = irpc::RcanEditor::new(id_client.clone());

    // get the auth systems
    let (hook, auth) = incoming(id_client, config.public());

    // make the endpoint
    let endpoint = Endpoint::builder(presets::N0)
        .secret_key(config.secret().clone())
        .hooks(hook)
        .bind()
        .await?;

    let router = RouterBuilder::new(endpoint.clone())
        .accept(auth::ALPN, auth)
        .accept(irpc::ALPN,rc_edit)
        .spawn();

    if let Some(target) = config.get_target() {
        if let Some(rcan) = config.get_rcan() {
            println!("{:#?}",caps::Caps::decode(rcan.clone().into_bytes()));
            let mut client = AuthClient::new(endpoint.clone(), target, rcan);
            let e = client.login().await;
            info!("{:?}", e);
            let cl = client.editor();
            println!("{:#?}",cl.info("fnord").await);
        } else {
            error!("No RCAN");
        }
    } else {
        error!("No Target");
    }

    let _ = tokio::signal::ctrl_c().await;
    info!("shutdown the router");
    let _ = router.shutdown().await;

    Ok(())
}
