/// A stack of caps, for authentication sets

use ed25519_dalek::VerifyingKey;
use rcan::{Capability, Rcan};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use anyhow::Result;

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

    pub fn check(&self,_verify_key: VerifyingKey)   { 
        // let capability = Caps::status();
        // let authorizer = rcan::Authorizer::new(verify_key);
        // let c: Vec<&Rcan<C>> = self.items.iter().map(|a| a).collect(); let proof_chain: &[&Rcan<C>]
        //let v = authorizer.check_invocation_from(invoker, capability, proof_chain);
    }

}
