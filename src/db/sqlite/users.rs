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

    async fn delete_user(&self, username: &str) -> Result<(), DataError> {
        sqlx::query(r#"DELETE FROM accounts WHERE user_id = ?"#)
            .bind(&username)
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
    use super::*;
    use crate::db::sqlite::Database;
    use crate::db::Users;

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
    async fn create_and_get_user_test() {
        let db = create_db().await;

        let insert_result = db
            .upsert_user(&User {
                username: "myuser".to_string(),
                password: "abc".to_string(),
            })
            .await;

        assert!(insert_result.is_ok());

        let user = db
            .get_user("myuser")
            .await
            .expect("User retrieval query failed");

        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.username, "myuser");
        assert_eq!(user.password, "abc");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn can_update_user() {
        let db = create_db().await;

        let insert_result1 = db
            .upsert_user(&User {
                username: "myuser".to_string(),
                password: "abc".to_string(),
            })
            .await;

        assert!(insert_result1.is_ok());

        let insert_result2 = db
            .upsert_user(&User {
                username: "myuser".to_string(),
                password: "123".to_string(),
            })
            .await;

        assert!(insert_result2.is_ok());

        let user = db
            .get_user("myuser")
            .await
            .expect("User retrieval query failed");

        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.username, "myuser");
        assert_eq!(user.password, "123"); //From second upsert
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn can_delete_user() {
        let db = create_db().await;

        let insert_result = db
            .upsert_user(&User {
                username: "myuser".to_string(),
                password: "abc".to_string(),
            })
            .await;

        assert!(insert_result.is_ok());

        db.delete_user("myuser")
            .await
            .expect("User deletion query failed");

        let user = db
            .get_user("myuser")
            .await
            .expect("User retrieval query failed");

        assert!(user.is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn username_not_in_db_returns_none() {
        let db = create_db().await;
        let user = db
            .get_user("does not exist")
            .await
            .expect("Get user query failure");

        assert!(user.is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn authenticate_user_is_some_with_valid_password() {
        let db = create_db().await;

        let insert_result = db
            .upsert_user(&User {
                username: "myuser".to_string(),
                password: crate::logic::hash_password("abc").expect("password hash error!"),
            })
            .await;

        assert!(insert_result.is_ok());

        let user = db
            .authenticate_user("myuser", "abc")
            .await
            .expect("User retrieval query failed");

        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.username, "myuser");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn authenticate_user_is_none_with_wrong_password() {
        let db = create_db().await;

        let insert_result = db
            .upsert_user(&User {
                username: "myuser".to_string(),
                password: crate::logic::hash_password("abc").expect("password hash error!"),
            })
            .await;

        assert!(insert_result.is_ok());

        let user = db
            .authenticate_user("myuser", "wrong-password")
            .await
            .expect("User retrieval query failed");

        assert!(user.is_none());
    }
}
