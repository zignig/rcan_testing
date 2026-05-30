// Some rcan capabilities

// Nicked from
// https://github.com/n0-computer/iroh-services/blob/main/src/caps.rs
//
// PERHAPS rewrite rcan.

use anyhow::Result;
use ed25519_dalek::VerifyingKey;
// use ed25519_dalek::pkcs8::spki::der::pem::decode;
use iroh::{EndpointId, PublicKey, SecretKey};
use rcan::{Capability, Expires, Rcan};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use core::slice::SlicePattern;
use std::{collections::BTreeSet, time::Duration};
use tracing::info;

/// A set of capabilities
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Serialize, Deserialize)]
pub struct CapSet<C: Capability + Ord>(BTreeSet<C>);

impl<C: Capability + Ord> Default for CapSet<C> {
    fn default() -> Self {
        Self(BTreeSet::new())
    }
}

impl<C: Capability + Ord> CapSet<C> {
    pub fn new(set: impl IntoIterator<Item = impl Into<C>>) -> Self {
        Self(BTreeSet::from_iter(set.into_iter().map(Into::into)))
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ C> + '_ {
        self.0.iter()
    }
}

impl<C: Capability + Ord> Capability for CapSet<C> {
    fn permits(&self, other: &Self) -> bool {
        other
            .iter()
            .all(|other_cap| self.iter().any(|self_cap| self_cap.permits(other_cap)))
    }
}

// The actual capability
#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Ord, Serialize, Deserialize)]
pub enum Cap {
    All,
    Info,
    Issue,
    Revoke,
    Status,
    PathTest { path: String },
}

impl Capability for Cap {
    fn permits(&self, other: &Self) -> bool {
        match (self, other) {
            (Cap::All, _) => true,
            (Cap::Info, Cap::Info) => true,
            (Cap::Issue, Cap::Issue) => true,
            (Cap::Revoke, Cap::Revoke) => true,
            (Cap::Status, Cap::Status) => true,
            (Cap::PathTest { path }, Cap::PathTest { path: otherpath }) => {
                self.path_check(path, otherpath)
            }
            (_, _) => false,
        }
    }
}

impl Cap {
    fn path_check(&self, source: &String, other: &String) -> bool {
        if !source.starts_with("/") {
            return false;
        }
        if !other.starts_with("/") {
            return false;
        }
        if other.starts_with(source) {
            return true;
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Caps {
    V0(CapSet<Cap>),
}

impl std::ops::Deref for Caps {
    type Target = CapSet<Cap>;

    fn deref(&self) -> &Self::Target {
        let Self::V0(slf) = self;
        slf
    }
}

impl Capability for Caps {
    fn permits(&self, other: &Self) -> bool {
        let Self::V0(slf) = self;
        let Self::V0(other) = other;
        slf.permits(other)
    }
}

impl Caps {
    pub fn new(caps: impl IntoIterator<Item = impl Into<Cap>>) -> Self {
        Self::V0(CapSet::new(caps))
    }

    pub fn all() -> Self {
        Self::new([Cap::All])
    }

    pub fn info() -> Self {
        Self::new([Cap::Info])
    }

    pub fn issue() -> Self {
        Self::new([
            Cap::Info,
            Cap::Issue,
            Cap::PathTest {
                path: "/hello".to_string(),
            },
        ])
    }

    pub fn status() -> Self {
        Self::new([Cap::Status])
    }

    pub fn as_text(&self) -> String {
        toml::to_string(self).unwrap()
    }

    pub fn make(
        &self,
        secret_key: &SecretKey,
        target: EndpointId,
        duration: Duration,
    ) -> Result<Rcan<Caps>> {
        let issuer = ed25519_dalek::SigningKey::from_bytes(&secret_key.to_bytes());
        let audience = target.as_verifying_key();
        let can = Rcan::issuing_builder(&issuer, audience, self.clone())
            .sign(Expires::valid_for(duration));
        // .sign(Expires::valid_for(Duration::from_secs(120)));
        Ok(can)
    }

    pub fn encoded(
        &self,
        secret_key: &SecretKey,
        target: EndpointId,
        duration: Duration,
    ) -> Result<String> {
        let rc = self.make(secret_key, target, duration)?;
        let ser = rc.encode();
        let mut encoded = data_encoding::BASE32_NOPAD.encode(&ser);
        encoded = encoded.to_lowercase();
        Ok(encoded)
    }

    pub fn decode(input: Vec<u8>) -> Result<Rcan<Caps>> {
        let upper = input.to_ascii_uppercase();
        let decoded = data_encoding::BASE32_NOPAD.decode(&upper)?;
        let deser = Rcan::<Caps>::decode(&decoded)?;
        Ok(deser)
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CapStack<C> {
    items: Vec<Rcan<C>>,
}

impl<C: Serialize + DeserializeOwned + Capability> CapStack<C> {
    pub fn new(first: Rcan<C>, second: Rcan<C>) -> Self {
        let mut v = Vec::new();
        v.push(first);
        v.push(second);
        Self { items: v }
    }

    pub fn encode(&self) -> Result<String> {
        let ser = postcard::to_allocvec(self)?;
        let mut encoded = data_encoding::BASE32_NOPAD.encode(&ser);
        encoded = encoded.to_lowercase();
        Ok(encoded)
    }

    pub fn decode(input: &[u8]) -> Result<Self> {
        println!("data string : {:?}", input);
        let upper = input.to_ascii_uppercase();
        let decoded = data_encoding::BASE32_NOPAD.decode(&upper)?;
        let can: Self = postcard::from_bytes(&decoded)?;
        Ok(can)
    }

    pub fn check(&self,verify_key: VerifyingKey)   { 
        let capability = Caps::status();
        let authorizer = rcan::Authorizer::new(verify_key);
        let c: Vec<&Rcan<C>> = self.items.iter().map(|a| a).collect(); let proof_chain: &[&Rcan<C>]
       
        
        //let v = authorizer.check_invocation_from(invoker, capability, proof_chain);
    }

}

pub fn issue(
    key: PublicKey,
    status: bool,
    all: bool,
    duration: Option<String>,
    secret: SecretKey,
) -> String {
    let dur = if let Some(duration) = duration {
        humantime::parse_duration(duration.as_str()).expect("Bad duration")
    } else {
        // 1 hour
        Duration::from_mins(60)
    };
    let cap = if status {
        Caps::status()
    } else {
        let mut c = Caps::issue();
        if all {
            c = Caps::info()
        }
        c
    };
    let rc = cap.encoded(&secret, key, dur).unwrap();
    info!("{:#?}", rc);
    rc
}
