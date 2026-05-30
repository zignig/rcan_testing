use ed25519_dalek::VerifyingKey;

use iroh::{EndpointId, PublicKey, SecretKey};
use std::path::PathBuf;

use n0_error::{AnyError, Result};

use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    secret: SecretKey,
    target: Option<EndpointId>,
    origin: Option<VerifyingKey>,
    rcan: Option<String>,
    #[serde(skip)]
    config_path: PathBuf,
}

impl Settings {
    pub fn load(config_path: PathBuf) -> Result<Settings, AnyError> {
        let config = match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                let content = content.as_str();
                let mut config: Settings = toml::from_str(&content).expect("config broken");
                // set my own config path
                config.config_path = config_path;
                config
            }
            Err(_e) => Settings::new(config_path),
        };
        Ok(config)
    }

    pub fn save(&self) {
        error!("{:#?}", self);
        let contents = toml::to_string(&self).expect("borked config");
        std::fs::write(self.config_path.clone(), contents).expect("borked file");
    }

    pub fn new(config_path: PathBuf) -> Settings {
        let secret = SecretKey::generate(&mut rand::rng());
        let set = Settings {
            secret,
            target: None,
            origin: None,
            rcan: None,
            config_path,
        };
        set.save();
        set
    }

    // pub fn set_ticket(&mut self, ticket: ServiceTicket) -> Result<()> {
    //     info!("Save a new ticket");
    //     println!("{:#?}", ticket);
    //     self.target = Some(ticket.target);
    //     self.origin = Some(ticket.origin);
    //     self.rcan = Some(ticket.rcan);
    //     self.save();
    //     Ok(())
    // }

    pub fn secret(&self) -> SecretKey {
        self.clone().secret
    }

    #[allow(dead_code)]
    pub fn origin(&self) -> Option<VerifyingKey> {
        self.clone().origin
    }

    pub fn public(&self) -> EndpointId {
        self.clone().secret.public()
    }

    pub fn get_target(&self) -> Option<PublicKey> {
        self.target.clone()
    }

    pub fn get_rcan(&self) -> Option<String> {
        self.rcan.clone()
    }
}
