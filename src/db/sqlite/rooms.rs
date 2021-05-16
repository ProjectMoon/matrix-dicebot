use super::errors::DataError;
use super::{Database, Rooms};
use crate::models::RoomInfo;
use async_trait::async_trait;
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

async fn record_event(conn: &SqlitePool, room_id: &str, event_id: &str) -> Result<(), DataError> {
    use std::convert::TryFrom;
    let now: i64 = i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock has gone backwards")
            .as_secs(),
    )?;

    sqlx::query(
        r#"INSERT INTO room_events
                      (room_id, event_id, event_timestamp)
                      VALUES (?, ?, ?)"#,
    )
    .bind(room_id)
    .bind(event_id)
    .bind(now)
    .execute(conn)
    .await?;

    Ok(())
}

#[async_trait]
impl Rooms for Database {
    async fn should_process(&self, room_id: &str, event_id: &str) -> Result<bool, DataError> {
        let row = sqlx::query!(
            r#"SELECT event_id FROM room_events
               WHERE room_id = ? AND event_id = ?"#,
            room_id,
            event_id
        )
        .fetch_optional(&self.conn)
        .await?;

        match row {
            Some(_) => Ok(false),
            None => {
                record_event(&self.conn, room_id, event_id).await?;
                Ok(true)
            }
        }
    }

    async fn insert_room_info(&self, info: &RoomInfo) -> Result<(), DataError> {
        sqlx::query(r#"INSERT INTO room_info (room_id, room_name) VALUES (?, ?)"#)
            .bind(&info.room_id)
            .bind(&info.room_name)
            .execute(&self.conn)
            .await?;

        Ok(())
    }

    async fn get_room_info(&self, room_id: &str) -> Result<Option<RoomInfo>, DataError> {
        let info = sqlx::query!(
            r#"SELECT room_id, room_name FROM room_info
                        WHERE room_id = ?"#,
            room_id
        )
        .fetch_optional(&self.conn)
        .await?;

        Ok(info.map(|i| RoomInfo {
            room_id: i.room_id,
            room_name: i.room_name,
        }))
    }

    async fn get_rooms_for_user(&self, user_id: &str) -> Result<HashSet<String>, DataError> {
        let room_ids = sqlx::query!(
            r#"SELECT room_id FROM room_users
                        WHERE username = ?"#,
            user_id
        )
        .fetch_all(&self.conn)
        .await?;

        Ok(room_ids.into_iter().map(|row| row.room_id).collect())
    }

    async fn get_users_in_room(&self, room_id: &str) -> Result<HashSet<String>, DataError> {
        let usernames = sqlx::query!(
            r#"SELECT username FROM room_users
                        WHERE room_id = ?"#,
            room_id
        )
        .fetch_all(&self.conn)
        .await?;

        Ok(usernames.into_iter().map(|row| row.username).collect())
    }

    async fn add_user_to_room(&self, username: &str, room_id: &str) -> Result<(), DataError> {
        self.remove_user_from_room(username, room_id).await.ok();

        sqlx::query("INSERT INTO room_users (room_id, username) VALUES (?, ?)")
            .bind(room_id)
            .bind(username)
            .execute(&self.conn)
            .await?;

        Ok(())
    }

    async fn remove_user_from_room(&self, username: &str, room_id: &str) -> Result<(), DataError> {
        sqlx::query("DELETE FROM room_users where username = ? AND room_id = ?")
            .bind(username)
            .bind(room_id)
            .execute(&self.conn)
            .await?;

        Ok(())
    }

    async fn clear_info(&self, room_id: &str) -> Result<(), DataError> {
        // We do not clear event history here, because if we rejoin a
        // room, we would re-process events we've already seen.
        sqlx::query("DELETE FROM room_info where room_id = ?")
            .bind(room_id)
            .execute(&self.conn)
            .await?;

        sqlx::query("DELETE FROM room_users where room_id = ?")
            .bind(room_id)
            .execute(&self.conn)
            .await?;

        Ok(())
    }
}
