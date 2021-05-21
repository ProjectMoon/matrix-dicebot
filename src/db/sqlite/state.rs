use super::Database;
use crate::db::{errors::DataError, DbState};
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
        // This will have to be updated if we ever add another column
        // to this table!
        sqlx::query("DELETE FROM bot_state")
            .execute(&self.conn)
            .await
            .ok();

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::sqlite::Database;
    use crate::db::DbState;

    async fn create_db() -> Database {
        let db_path = tempfile::NamedTempFile::new_in(".").unwrap();
        crate::db::sqlite::migrator::migrate(db_path.path().to_str().unwrap())
            .await
            .unwrap();

        Database::new(db_path.path().to_str().unwrap())
            .await
            .unwrap()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn set_and_get_device_id() {
        let db = create_db().await;

        db.set_device_id("device_id")
            .await
            .expect("Could not set device ID");

        let device_id = db.get_device_id().await.expect("Could not get device ID");

        assert!(device_id.is_some());
        assert_eq!(device_id.unwrap(), "device_id");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn no_device_id_set_returns_none() {
        let db = create_db().await;
        let device_id = db.get_device_id().await.expect("Could not get device ID");
        assert!(device_id.is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn can_update_device_id() {
        let db = create_db().await;

        db.set_device_id("device_id")
            .await
            .expect("Could not set device ID");

        db.set_device_id("device_id2")
            .await
            .expect("Could not set device ID");

        let device_id = db.get_device_id().await.expect("Could not get device ID");

        assert!(device_id.is_some());
        assert_eq!(device_id.unwrap(), "device_id2");
    }
}
