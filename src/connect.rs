// Client tools for connecting to the keyparty service

use anyhow::{Error, Result};
use iroh::{Endpoint, EndpointId};
use n0_error::anyerr;
use tokio::io::AsyncReadExt;
use tracing::{debug, error};

use crate::{auth::ALPN as AUTH_ALPN, irpc::RcanClient};

/// This is a client to connect to a share signer
pub struct AuthClient {
    endpoint: Endpoint,
    target: EndpointId,
    rcan: String,
    authed: bool,
}

impl AuthClient {
    /// Create a new client
    pub fn new(endpoint: Endpoint, target: EndpointId, rcan: String) -> Self {
        Self {
            endpoint,
            target,
            rcan,
            authed: false,
        }
    }

    // The editor
    pub fn editor(&self) -> RcanClient {
        RcanClient::connect(self.endpoint.clone(), self.target)
    }
    /// Is the client connected
    pub fn connected(&self) -> bool {
        self.authed
    }

    /// Login to the remote service.
    pub async fn login(&mut self) -> Result<(), Error> {
        let mut counter = 0;
        const MAX_FAIL: i32 = 5;
        loop {
            match self.auth().await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    counter += 1;
                    if counter == MAX_FAIL {
                        error!("{:#?} - {} ", e, counter);
                        return Err(e.into());
                    }
                }
            };
        }
    }

    /// Send the rcan up to the service client.
    /// This gets cached on the server so you only have to ask once.
    pub async fn auth(&mut self) -> Result<()> {
        debug!("endpoint auth send {}", self.target.fmt_short());
        let conn = self.endpoint.connect(self.target, AUTH_ALPN).await?;

        debug!("auth incoming");
        let (mut send, mut recv) = conn.open_bi().await?;

        // send the rcan up
        let buf = self.rcan.clone().into_bytes();

        // write
        let _sent = send.write(&buf).await?;
        // info!("send {} bytes", sent);
        send.finish()?;

        // get the response
        let msg = recv.read_u8().await?;
        conn.close(1u8.into(), b"finished");
        debug!("reply message {:?}", msg);
        if msg == 1 {
            self.authed = true;
            return Ok(());
        } else {
            return Err(anyerr!("auth failed").into());
        }
    }
}
