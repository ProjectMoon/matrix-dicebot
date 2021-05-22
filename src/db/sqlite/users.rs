use super::Database;
use crate::db::{errors::DataError, Users};
use crate::error::BotError;
use crate::models::User;
use async_trait::async_trait;

#[async_trait]
impl Users for Database {
    async fn upsert_user(&self, user: &User) -> Result<(), DataError> {
        sqlx::query(
            r#"INSERT INTO accounts (user_id, password) VALUES (?, ?)
               ON CONFLICT(user_id) DO UPDATE SET password = ?"#,
        )
        .bind(&user.username)
        .bind(&user.password)
        .bind(&user.password)
        .execute(&self.conn)
        .await?;

        Ok(())
    }

    async fn get_user(&self, username: &str) -> Result<Option<User>, DataError> {
        let user_row = sqlx::query!(
            r#"SELECT user_id, password FROM accounts
               WHERE user_id = ?"#,
            username
        )
        .fetch_optional(&self.conn)
        .await?;

        Ok(user_row.map(|u| User {
            username: u.user_id,
            password: u.password,
        }))
    }

    async fn authenticate_user(
        &self,
        username: &str,
        raw_password: &str,
    ) -> Result<Option<User>, BotError> {
        let user = self.get_user(username).await?;
        Ok(user.filter(|u| u.verify_password(raw_password)))
    }
}

#[cfg(test)]
mod tests {
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
