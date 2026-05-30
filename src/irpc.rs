// IRPC interface

pub const ALPN: &[u8] = b"rcan/editor/0";

use iroh::{
    Endpoint,
    protocol::{AcceptError, ProtocolHandler},
};
use irpc_iroh::{IrohLazyRemoteConnection, read_request};
use serde::{Deserialize, Serialize};

use irpc::{Client, WithChannels, channel::oneshot, rpc_requests};

use tracing::{error, info};

use crate::IdClient;

// Irpc structs
#[derive(Debug, Serialize, Deserialize)]
struct Info {
    data: String,
}

#[rpc_requests(message = SigningMessage)]
#[derive(Serialize, Deserialize, Debug)]
enum RcanEditProtocol {
    #[rpc(tx=oneshot::Sender<Result<String,String>>)]
    Info(Info),
}

#[derive(Debug)]
pub struct RcanEditor {
    id_client: IdClient,
}

impl RcanEditor {
    pub fn new(id_client: IdClient) -> Self {
        Self { id_client }
    }
}

impl ProtocolHandler for RcanEditor {
    async fn accept(&self, conn: iroh::endpoint::Connection) -> Result<(), AcceptError> {
        // Check if the rcan is still valid
        let id = conn.remote_id();
        if self.id_client.check(id).await.expect("rcan expired") {
            info!("rcan good for {}", id.fmt_short());
        } else {
            conn.close(1u32.into(), b"invalid message");
            return Ok(());
        };
        let fren = self.id_client.get(conn.remote_id()).await.unwrap().unwrap();
        while let Some(msg) = read_request::<RcanEditProtocol>(&conn).await? {
            match msg {
                SigningMessage::Info(msg) => {
                    let WithChannels { inner, tx, .. } = msg;
                    // Check to see if the rcan has the correct powers;
                    if fren.can_info() {
                        // Send to the signer
                        let val = inner.data;
                        info!(" {:#?}", val);
                        tx.send(Ok(val)).await.ok();
                    } else {
                        error!("Rcan Does not have info");
                        tx.send(Err("No info permission".to_string()))
                            .await
                            .ok();
                    }
                }
            }
        }
        info!("Client {} disconnectd", conn.remote_id().fmt_short());
        Ok(())
    }
}

pub struct RcanClient {
    inner: Client<RcanEditProtocol>,
}

impl RcanClient {
    pub fn connect(endpoint: Endpoint, addr: impl Into<iroh::EndpointAddr>) -> Self {
        let conn = IrohLazyRemoteConnection::new(endpoint, addr.into(), ALPN.to_vec());
        Self {
            inner: Client::boxed(conn),
        }
    }

    //TODO fix the result stack
    pub async fn info(&self, data: &str) -> Result<String, anyhow::Error> {
        self.inner
            .rpc(Info {
                data: data.to_string(),
            })
            .await?
            .map_err(|err| anyhow::anyhow!(err))
    }
}
