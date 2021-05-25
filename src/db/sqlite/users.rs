use super::Database;
use crate::db::{errors::DataError, Users};
use crate::error::BotError;
use crate::models::User;
use async_trait::async_trait;
use log::info;

#[async_trait]
impl Users for Database {
    async fn upsert_user(&self, user: &User) -> Result<(), DataError> {
        let mut tx = self.conn.begin().await?;

        sqlx::query!(
            r#"INSERT INTO accounts (user_id, password, account_status)
               VALUES (?, ?, ?)
               ON CONFLICT(user_id) DO
               UPDATE SET password = ?, account_status = ?"#,
            user.username,
            user.password,
            user.account_status,
            user.password,
            user.account_status
        )
        .execute(&mut tx)
        .await?;

        sqlx::query!(
            r#"INSERT INTO user_state (user_id, active_room)
               VALUES (?, ?)
               ON CONFLICT(user_id) DO
               UPDATE SET active_room = ?"#,
            user.username,
            user.active_room,
            user.active_room
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn delete_user(&self, username: &str) -> Result<(), DataError> {
        let mut tx = self.conn.begin().await?;

        sqlx::query!(r#"DELETE FROM accounts WHERE user_id = ?"#, username)
            .execute(&mut tx)
            .await?;

        sqlx::query!(r#"DELETE FROM user_state WHERE user_id = ?"#, username)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn get_user(&self, username: &str) -> Result<Option<User>, DataError> {
        // Should be query_as! macro, but the left join breaks it with a
        // non existing error message.
        let user_row: Option<User> = sqlx::query_as(
            r#"SELECT
               a.user_id as "username",
               a.password,
               s.active_room,
               COALESCE(a.account_status, 'not_registered') as "account_status"
               FROM accounts a
               LEFT JOIN user_state s on a.user_id = s.user_id
               WHERE a.user_id = ?"#,
        )
        .bind(username)
        .fetch_optional(&self.conn)
        .await?;

        Ok(user_row)
    }

    //TODO should this logic be moved further up into logic.rs maybe?
    async fn get_or_create_user(&self, username: &str) -> Result<User, DataError> {
        let maybe_user = self.get_user(username).await?;

        match maybe_user {
            Some(user) => Ok(user),
            None => {
                info!("Creating unregistered account for {}", username);
                let user = User::unregistered(&username);
                self.upsert_user(&user).await?;
                Ok(user)
            }
        }
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
    use crate::models::AccountStatus;

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
    async fn get_or_create_user_no_user_exists() {
        let db = create_db().await;

        let user = db
            .get_or_create_user("@test:example.com")
            .await
            .expect("User creation didn't work.");

        assert_eq!(user.username, "@test:example.com");

        let user_again = db
            .get_user("@test:example.com")
            .await
            .expect("User retrieval didn't work.")
            .expect("No user returned from option.");

        assert_eq!(user, user_again);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn get_or_create_user_when_user_exists() {
        let db = create_db().await;

        let user = User {
            username: "myuser".to_string(),
            password: Some("abc".to_string()),
            account_status: AccountStatus::Registered,
            active_room: Some("myroom".to_string()),
        };

        let insert_result = db.upsert_user(&user).await;
        assert!(insert_result.is_ok());

        let user_again = db
            .get_or_create_user("myuser")
            .await
            .expect("User retrieval didn't work.");

        assert_eq!(user, user_again);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn create_and_get_full_user_test() {
        let db = create_db().await;

        let insert_result = db
            .upsert_user(&User {
                username: "myuser".to_string(),
                password: Some("abc".to_string()),
                account_status: AccountStatus::Registered,
                active_room: Some("myroom".to_string()),
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
        assert_eq!(user.password, Some("abc".to_string()));
        assert_eq!(user.account_status, AccountStatus::Registered);
        assert_eq!(user.active_room, Some("myroom".to_string()));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn can_get_user_with_no_state_record() {
        let db = create_db().await;

        let insert_result = db
            .upsert_user(&User {
                username: "myuser".to_string(),
                password: Some("abc".to_string()),
                account_status: AccountStatus::AwaitingActivation,
                active_room: Some("myroom".to_string()),
            })
            .await;

        assert!(insert_result.is_ok());

        sqlx::query("DELETE FROM user_state")
            .execute(&db.conn)
            .await
            .expect("Could not delete from user_state table.");

        let user = db
            .get_user("myuser")
            .await
            .expect("User retrieval query failed");

        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.username, "myuser");
        assert_eq!(user.password, Some("abc".to_string()));
        assert_eq!(user.account_status, AccountStatus::AwaitingActivation);

        //These should be default values because the state record is missing.
        assert_eq!(user.active_room, None);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn can_insert_without_password() {
        let db = create_db().await;

        let insert_result = db
            .upsert_user(&User {
                username: "myuser".to_string(),
                password: None,
                ..Default::default()
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
        assert_eq!(user.password, None);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn can_insert_without_active_room() {
        let db = create_db().await;

        let insert_result = db
            .upsert_user(&User {
                username: "myuser".to_string(),
                active_room: None,
                ..Default::default()
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
        assert_eq!(user.active_room, None);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn can_update_user() {
        let db = create_db().await;

        let insert_result1 = db
            .upsert_user(&User {
                username: "myuser".to_string(),
                password: Some("abc".to_string()),
                ..Default::default()
            })
            .await;

        assert!(insert_result1.is_ok());

        let insert_result2 = db
            .upsert_user(&User {
                username: "myuser".to_string(),
                password: Some("123".to_string()),
                active_room: Some("room".to_string()),
                account_status: AccountStatus::AwaitingActivation,
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

        //From second upsert
        assert_eq!(user.password, Some("123".to_string()));
        assert_eq!(user.active_room, Some("room".to_string()));
        assert_eq!(user.account_status, AccountStatus::AwaitingActivation);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn can_delete_user() {
        let db = create_db().await;

        let insert_result = db
            .upsert_user(&User {
                username: "myuser".to_string(),
                password: Some("abc".to_string()),
                ..Default::default()
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
                password: Some(crate::logic::hash_password("abc").expect("password hash error!")),
                ..Default::default()
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
                password: Some(crate::logic::hash_password("abc").expect("password hash error!")),
                ..Default::default()
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
