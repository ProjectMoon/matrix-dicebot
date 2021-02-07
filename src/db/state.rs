use crate::db::errors::DataError;
use sled::Tree;

#[derive(Clone)]
pub struct DbState {
    /// Tree of simple key-values for global state values that persist
    /// between restarts (e.g. device ID).
    pub(in crate::db) global_metadata: Tree,
}

const DEVICE_ID_KEY: &'static [u8] = b"device_id";

impl DbState {
    pub(in crate::db) fn new(db: &sled::Db) -> Result<DbState, sled::Error> {
        Ok(DbState {
            global_metadata: db.open_tree("global_metadata")?,
        })
    }

    pub fn get_device_id(&self) -> Result<Option<String>, DataError> {
        self.global_metadata
            .get(DEVICE_ID_KEY)?
            .map(|v| String::from_utf8(v.to_vec()))
            .transpose()
            .map_err(|e| e.into())
    }

    pub fn set_device_id(&self, device_id: &str) -> Result<(), DataError> {
        self.global_metadata
            .insert(DEVICE_ID_KEY, device_id.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sled::Config;

    fn create_test_instance() -> DbState {
        let config = Config::new().temporary(true);
        let db = config.open().unwrap();
        DbState::new(&db).unwrap()
    }

    #[test]
    fn set_device_id_works() {
        let state = create_test_instance();
        let result = state.set_device_id("test-device");
        assert!(result.is_ok());
    }

    #[test]
    fn set_device_id_can_overwrite() {
        let state = create_test_instance();
        state.set_device_id("test-device").expect("insert 1 failed");
        let result = state.set_device_id("test-device2");
        assert!(result.is_ok());
    }

    #[test]
    fn get_device_id_returns_some_when_set() {
        let state = create_test_instance();

        state
            .set_device_id("test-device")
            .expect("could not store device id properly");

        let device_id = state.get_device_id();

        assert!(device_id.is_ok());

        let device_id = device_id.unwrap();
        assert!(device_id.is_some());
        assert_eq!("test-device", device_id.unwrap());
    }

    #[test]
    fn get_device_id_returns_none_when_unset() {
        let state = create_test_instance();
        let device_id = state.get_device_id();
        assert!(device_id.is_ok());

        let device_id = device_id.unwrap();
        assert!(device_id.is_none());
    }
}
