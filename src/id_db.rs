// Data base redb for fren
// pub struct Database;

use n0_error::Result;
use postcard::to_stdvec;
use redb::{Database, TableDefinition, TypeName, Value};
use std::path::PathBuf;

use crate::id_store::Fren;

// Database
const NODE_TABLE: TableDefinition<&[u8; 32], Fren> = TableDefinition::new("nodes");

// KV impl
impl Value for Fren {
    type SelfType<'a>
        = Fren
    where
        Self: 'a;

    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        postcard::from_bytes(data).unwrap()
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        to_stdvec(value).unwrap()
    }

    fn type_name() -> redb::TypeName {
        TypeName::new("Fren")
    }
}

pub struct IDdatabase {
    path: PathBuf,
    db: Database,
}

impl IDdatabase {
    pub fn new(path: PathBuf) -> Result<Self> {
        let db = Database::create(&path).expect("database can't be opened");
        // Create the tables
        let write_txn = db.begin_write().unwrap();
        let _ = write_txn.open_table(NODE_TABLE).unwrap();
        write_txn.commit().unwrap();
        
        Ok(Self { path, db })
    }
}
