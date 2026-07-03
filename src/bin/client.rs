/// Rcan testing client
///
use clap::Parser;
use iroh::Endpoint;
use iroh::endpoint::presets;
use iroh::protocol::RouterBuilder;
use n0_error::Result;

use rcan_testing::{
    Args, IdentityApi, Settings, auth,
    capset::{self, Caps},
    capstack::CapStack,
    cli::Command,
    connect::AuthClient,
    incoming, irpc, repl,
};

use tracing::info;
use tracing::error;
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut filter = Targets::new();
    filter = filter.with_target("rcan_testing", LevelFilter::DEBUG);
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    // Arguments
    let args = Args::parse();
    // info!("cmd line {:#?}", args);

    // Settings
    let config = Settings::load(args.config)?;
    // info!("{:#?}", config);

    println!("id {}", config.public());

    // Create the identity client
    let id_service = IdentityApi::new(config.get_database()).await;
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
            println!("{:#?}", rc_obj);
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
            
            let mut rpl = repl::make_repl(cl).await.expect("repl broken");
            let _ = rpl.run_async().await;

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
