use crate::db::errors::DataError;
use crate::db::variables::Variables;
use sled::Db;
use std::path::Path;

pub mod errors;
pub mod schema;
pub mod variables;

#[derive(Clone)]
pub struct Database {
    db: Db,
    pub(crate) variables: Variables,
    //rooms: Tree,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Database, DataError> {
        let db = sled::open(path)?;
        let variables = db.open_tree("variables")?;
        //let rooms = db.open_tree("rooms")?;

        Ok(Database {
            db: db.clone(),
            variables: Variables(variables),
            //rooms: rooms,
        })
    }
}
