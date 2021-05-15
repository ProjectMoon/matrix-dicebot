use super::errors::DataError;
use super::{Database, DbState};
use async_trait::async_trait;

#[async_trait]
impl DbState for Database {
    async fn get_device_id(&self) -> Result<Option<String>, DataError> {
        Ok(None)
    }

    async fn set_device_id(&self, device_id: &str) -> Result<(), DataError> {
        Ok(())
    }
}
