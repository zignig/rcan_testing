/// Rcan testing client
///
use clap::Parser;
use iroh::Endpoint;
use iroh::endpoint::presets;
use iroh::protocol::RouterBuilder;
use n0_error::Result;

use rcan_testing::auth;
use rcan_testing::capset::{self, Caps};
use rcan_testing::capstack::CapStack;
use rcan_testing::cli::Command;
use rcan_testing::connect::AuthClient;
use rcan_testing::incoming;
use rcan_testing::irpc;
use rcan_testing::{Args, IdentityApi, Settings};

use tracing::error;
use tracing::info;

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

    if let Some(command) = args.command {
        match command {
            Command::Issue {
                key,
                status,
                all,
                duration,
            } => {
                let _cap = capset::issue(key, status, all, duration, config.secret());
            }
        }
    }

    // the rcan editor
    let rc_edit = irpc::RcanEditor::new(id_client.clone());

    // get the auth systems
    let (hook, auth) = incoming(id_client.clone(), config.public());

    // make the endpoint
    let endpoint = Endpoint::builder(presets::N0)
        .secret_key(config.secret().clone())
        .hooks(hook)
        .bind()
        .await?;

    let router = RouterBuilder::new(endpoint.clone())
        .accept(auth::ALPN, auth)
        .accept(irpc::ALPN, rc_edit)
        .spawn();

    if let Some(target) = config.get_target() {
        if let Some(rcan) = config.get_rcan() {
            // insert the target into the local id store
            let rc_obj = Caps::decode(rcan.clone().into_bytes()).expect("bad rcan");
            id_client.new_fren(target, rc_obj.clone()).await;
            // println!("{:#?}", rc_obj);
            // test out the cap stack.
            if args.test {
                println!("woohoo !!");
                let cs = CapStack::new(rc_obj.clone(), rc_obj);
                let st = cs.encode().expect("bad rcan");
                println!("{:#?}", &st);
                let cs2 = CapStack::<Caps>::decode(st.as_bytes());
                println!("again {:#?}", cs2);
            }

            let mut client = AuthClient::new(endpoint.clone(), target, rcan);
            let e = client.login().await;
            info!("{:?}", e);
            let cl = client.editor();
            println!("{:#?}", cl.info("fnord").await);
            let _ = router.shutdown().await;
            return Ok(());
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
