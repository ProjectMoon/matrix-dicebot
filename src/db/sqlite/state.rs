use super::errors::DataError;
use super::{Database, DbState};
use async_trait::async_trait;

#[async_trait]
impl DbState for Database {
    async fn get_device_id(&self) -> Result<Option<String>, DataError> {
        let state = sqlx::query!(r#"SELECT device_id FROM bot_state limit 1"#)
            .fetch_optional(&self.conn)
            .await?;

        Ok(state.map(|s| s.device_id))
    }

    async fn set_device_id(&self, device_id: &str) -> Result<(), DataError> {
        sqlx::query(
            r#"INSERT INTO bot_state
                      (device_id)
                      VALUES (?)"#,
        )
        .bind(device_id)
        .execute(&self.conn)
        .await?;

        Ok(())
    }
}
