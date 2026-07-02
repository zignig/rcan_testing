// turso database with toasty

use std::path::PathBuf;
use anyhow::{anyhow,Result};
use iroh::PublicKey;

#[derive(Debug, PartialEq, toasty::Embed)]
pub enum EndPointStatus {
    #[column(variant = 1)]
    Pending,
    #[column(variant = 2)]
    Active,
    #[column(variant = 3)]
    Done,
}

#[derive(Debug, toasty::Model)]
pub struct FrenProxy {
    #[key]
    id: String,
    status: EndPointStatus,
    nickname: Option<String>,
    secret: Option<Vec<u8>>,
}

// The pub struct
pub struct PersistStore {
    db: toasty::Db,
}

impl PersistStore {
    pub async fn new(path: PathBuf) -> Result<Self> {
        if let Some(path) = path.to_str() {
            let conn = format!("turso:{}", path);
            let db = toasty::Db::builder()
                .models(toasty::models!(crate::*))
                .connect(&conn)
                .await?;
            let _ = db.push_schema().await;
            return Ok(Self { db });
        } else {
            return Err(anyhow!("bad db string"));
        }
    }

    pub async fn add(&mut self, pk: PublicKey) -> Result<FrenProxy> {
        let ep = toasty::create!(FrenProxy {
            id: pk.to_string(),
            status: EndPointStatus::Pending
        })
        .exec(&mut self.db)
        .await?;
        Ok(ep)
    }

    pub async fn all(&mut self) -> Vec<FrenProxy> {
        FrenProxy::all().exec(&mut self.db).await.expect("fail find all")
    }

    pub async fn get(&mut self, pk: PublicKey) -> Option<FrenProxy> {
        let pks = pk.to_string();
        match FrenProxy::get_by_id(&mut self.db, pks).await {
            Ok(fren) => Some(fren),
            Err(_) => None,
        }
    }
}