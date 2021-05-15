use super::errors::DataError;
use super::{Database, Variables};
use async_trait::async_trait;
use std::collections::HashMap;

struct UserVariableRow {
    key: String,
    value: i32,
}

#[async_trait]
impl Variables for Database {
    async fn get_user_variables(
        &self,
        user: &str,
        room_id: &str,
    ) -> Result<HashMap<String, i32>, DataError> {
        let rows = sqlx::query!(
            r#"SELECT key, value as "value: i32" FROM user_variables
               WHERE room_id = ?"#,
            room_id,
        )
        .fetch_all(&self.conn)
        .await?;

        Ok(rows.into_iter().map(|row| (row.key, row.value)).collect())
    }

    async fn get_variable_count(&self, user: &str, room_id: &str) -> Result<i32, DataError> {
        Ok(1)
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
        .fetch_one(&self.conn)
        .await?;

        Ok(row.value)
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
                    (user_id, room_id, variable_name, value)
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
             WHERE user_id = ? AND room_id = ? AND variable_name = ?",
        )
        .bind(user)
        .bind(room_id)
        .bind(variable_name)
        .execute(&self.conn)
        .await?;

        Ok(())
    }
}
