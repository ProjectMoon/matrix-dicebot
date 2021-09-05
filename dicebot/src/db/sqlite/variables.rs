use super::Database;
use crate::db::{errors::DataError, Variables};
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
impl Variables for Database {
    async fn get_user_variables(
        &self,
        user: &str,
        room_id: &str,
    ) -> Result<HashMap<String, i32>, DataError> {
        let rows = sqlx::query!(
            r#"SELECT key, value as "value: i32" FROM user_variables
               WHERE room_id = ? AND user_id = ?"#,
            room_id,
            user
        )
        .fetch_all(&self.conn)
        .await?;

        Ok(rows.into_iter().map(|row| (row.key, row.value)).collect())
    }

    async fn get_variable_count(&self, user: &str, room_id: &str) -> Result<i32, DataError> {
        let row = sqlx::query!(
            r#"SELECT count(*) as "count: i32" FROM user_variables
               WHERE room_id = ? and user_id = ?"#,
            room_id,
            user
        )
        .fetch_optional(&self.conn)
        .await?;

        Ok(row.map(|r| r.count).unwrap_or(0))
    }

    async fn get_user_variable(
        &self,
        user: &str,
        room_id: &str,
        variable_name: &str,
    ) -> Result<i32, DataError> {
        let row = sqlx::query!(
            r#"SELECT value as "value: i32" FROM user_variables
               WHERE user_id = ? AND room_id = ? AND key = ?"#,
            user,
            room_id,
            variable_name
        )
        .fetch_optional(&self.conn)
        .await?;

        row.map(|r| r.value)
            .ok_or_else(|| DataError::KeyDoesNotExist(variable_name.to_string()))
    }

    async fn set_user_variable(
        &self,
        user: &str,
        room_id: &str,
        variable_name: &str,
        value: i32,
    ) -> Result<(), DataError> {
        sqlx::query(
            "INSERT INTO user_variables
                    (user_id, room_id, key, value)
                    values (?, ?, ?, ?)",
        )
        .bind(user)
        .bind(room_id)
        .bind(variable_name)
        .bind(value)
        .execute(&self.conn)
        .await?;

        Ok(())
    }

    async fn delete_user_variable(
        &self,
        user: &str,
        room_id: &str,
        variable_name: &str,
    ) -> Result<(), DataError> {
        sqlx::query(
            "DELETE FROM user_variables
             WHERE user_id = ? AND room_id = ? AND key = ?",
        )
        .bind(user)
        .bind(room_id)
        .bind(variable_name)
        .execute(&self.conn)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::sqlite::Database;
    use crate::db::Variables;
    use std::future::Future;

    async fn with_db<Fut>(f: impl FnOnce(Database) -> Fut)
    where
        Fut: Future<Output = ()>,
    {
        let db_path = tempfile::NamedTempFile::new_in(".").unwrap();
        crate::db::sqlite::migrator::migrate(db_path.path().to_str().unwrap())
            .await
            .unwrap();

        let db = Database::new(db_path.path().to_str().unwrap())
            .await
            .unwrap();

        f(db).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn set_and_get_variable_test() {
        with_db(|db| async move {
            db.set_user_variable("myuser", "myroom", "myvariable", 1)
                .await
                .expect("Could not set variable");

            let value = db
                .get_user_variable("myuser", "myroom", "myvariable")
                .await
                .expect("Could not get variable");

            assert_eq!(value, 1);
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn get_missing_variable_test() {
        with_db(|db| async move {
            let value = db.get_user_variable("myuser", "myroom", "myvariable").await;

            assert!(value.is_err());
            assert!(matches!(
                value.err().unwrap(),
                DataError::KeyDoesNotExist(_)
            ));
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn get_other_user_variable_test() {
        with_db(|db| async move {
            db.set_user_variable("myuser1", "myroom", "myvariable", 1)
                .await
                .expect("Could not set variable");

            let value = db
                .get_user_variable("myuser2", "myroom", "myvariable")
                .await;

            assert!(value.is_err());
            assert!(matches!(
                value.err().unwrap(),
                DataError::KeyDoesNotExist(_)
            ));
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn count_variables_test() {
        with_db(|db| async move {
            for variable_name in &["var1", "var2", "var3"] {
                db.set_user_variable("myuser", "myroom", variable_name, 1)
                    .await
                    .expect("Could not set variable");
            }

            let count = db
                .get_variable_count("myuser", "myroom")
                .await
                .expect("Could not get count.");

            assert_eq!(count, 3);
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn count_variables_respects_user_id() {
        with_db(|db| async move {
            for variable_name in &["var1", "var2", "var3"] {
                db.set_user_variable("different-user", "myroom", variable_name, 1)
                    .await
                    .expect("Could not set variable");
            }

            let count = db
                .get_variable_count("myuser", "myroom")
                .await
                .expect("Could not get count.");

            assert_eq!(count, 0);
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn count_variables_respects_room_id() {
        with_db(|db| async move {
            for variable_name in &["var1", "var2", "var3"] {
                db.set_user_variable("myuser", "different-room", variable_name, 1)
                    .await
                    .expect("Could not set variable");
            }

            let count = db
                .get_variable_count("myuser", "myroom")
                .await
                .expect("Could not get count.");

            assert_eq!(count, 0);
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn delete_variable_test() {
        with_db(|db| async move {
            for variable_name in &["var1", "var2", "var3"] {
                db.set_user_variable("myuser", "myroom", variable_name, 1)
                    .await
                    .expect("Could not set variable");
            }

            db.delete_user_variable("myuser", "myroom", "var1")
                .await
                .expect("Could not delete variable.");

            let count = db
                .get_variable_count("myuser", "myroom")
                .await
                .expect("Could not get count");

            assert_eq!(count, 2);

            let var1 = db.get_user_variable("myuser", "myroom", "var1").await;
            assert!(var1.is_err());
            assert!(matches!(var1.err().unwrap(), DataError::KeyDoesNotExist(_)));
        })
        .await;
    }
}
