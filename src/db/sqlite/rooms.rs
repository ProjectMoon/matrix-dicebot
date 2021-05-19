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
        //Clear out old info first, because we want this to be an "upsert."
        sqlx::query("DELETE FROM room_info where room_id = ?")
            .bind(&info.room_id)
            .execute(&self.conn)
            .await
            .ok();

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
        // This is here because it is possible to process a bunch of
        // user join/leave events at once, and we don't want to cause
        // constraint violation errors.
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

#[cfg(test)]
mod tests {
    use super::super::Rooms;
    use super::*;

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
    async fn should_process_test() {
        let db = create_db().await;

        let first_check = db
            .should_process("myroom", "myeventid")
            .await
            .expect("should_process failed in first insert");

        assert_eq!(first_check, true);

        let second_check = db
            .should_process("myroom", "myeventid")
            .await
            .expect("should_process failed in first insert");

        assert_eq!(second_check, false);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn insert_and_get_room_info_test() {
        let db = create_db().await;

        let info = RoomInfo {
            room_id: "myroomid".to_string(),
            room_name: "myroomname".to_string(),
        };

        db.insert_room_info(&info)
            .await
            .expect("Could not insert room info.");

        let retrieved_info = db
            .get_room_info("myroomid")
            .await
            .expect("Could not retrieve room info.");

        assert!(retrieved_info.is_some());
        assert_eq!(info, retrieved_info.unwrap());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn insert_room_info_updates_existing() {
        let db = create_db().await;

        let info1 = RoomInfo {
            room_id: "myroomid".to_string(),
            room_name: "myroomname".to_string(),
        };

        db.insert_room_info(&info1)
            .await
            .expect("Could not insert room info1.");

        let info2 = RoomInfo {
            room_id: "myroomid".to_string(),
            room_name: "myroomname2".to_string(),
        };

        db.insert_room_info(&info2)
            .await
            .expect("Could not update room info after first insert");

        let retrieved_info = db
            .get_room_info("myroomid")
            .await
            .expect("Could not get room info");

        assert!(retrieved_info.is_some());
        let retrieved_info = retrieved_info.unwrap();

        assert_eq!(retrieved_info.room_id, "myroomid");
        assert_eq!(retrieved_info.room_name, "myroomname2");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn add_user_to_room_test() {
        let db = create_db().await;

        db.add_user_to_room("myuser", "myroom")
            .await
            .expect("Could not add user to room.");

        let users_in_room = db
            .get_users_in_room("myroom")
            .await
            .expect("Could not get users in room.");

        assert_eq!(users_in_room.len(), 1);
        assert!(users_in_room.contains("myuser"));

        let rooms_for_user = db
            .get_rooms_for_user("myuser")
            .await
            .expect("Could not get rooms for user");

        assert_eq!(rooms_for_user.len(), 1);
        assert!(rooms_for_user.contains("myroom"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn add_user_to_room_does_not_have_constraint_violation() {
        let db = create_db().await;

        db.add_user_to_room("myuser", "myroom")
            .await
            .expect("Could not add user to room.");

        let second_attempt = db.add_user_to_room("myuser", "myroom").await;

        assert!(second_attempt.is_ok());

        let users_in_room = db
            .get_users_in_room("myroom")
            .await
            .expect("Could not get users in room.");

        assert_eq!(users_in_room.len(), 1);
        assert!(users_in_room.contains("myuser"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn remove_user_from_room_test() {
        let db = create_db().await;

        db.add_user_to_room("myuser", "myroom")
            .await
            .expect("Could not add user to room.");

        let remove_attempt = db.remove_user_from_room("myuser", "myroom").await;

        assert!(remove_attempt.is_ok());

        let users_in_room = db
            .get_users_in_room("myroom")
            .await
            .expect("Could not get users in room.");

        assert_eq!(users_in_room.len(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn clear_info_does_not_delete_other_rooms() {
        let db = create_db().await;

        let info1 = RoomInfo {
            room_id: "myroomid".to_string(),
            room_name: "myroomname".to_string(),
        };

        let info2 = RoomInfo {
            room_id: "myroomid2".to_string(),
            room_name: "myroomname2".to_string(),
        };

        db.insert_room_info(&info1)
            .await
            .expect("Could not insert room info1.");

        db.insert_room_info(&info2)
            .await
            .expect("Could not insert room info2.");

        db.add_user_to_room("myuser", &info1.room_id)
            .await
            .expect("Could not add user to room.");

        db.clear_info(&info1.room_id)
            .await
            .expect("Could not clear room info1");

        let room_info2 = db
            .get_room_info(&info2.room_id)
            .await
            .expect("Could not get room info2.");

        assert!(room_info2.is_some());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn clear_info_test() {
        let db = create_db().await;

        let info = RoomInfo {
            room_id: "myroomid".to_string(),
            room_name: "myroomname".to_string(),
        };

        db.insert_room_info(&info)
            .await
            .expect("Could not insert room info.");

        db.add_user_to_room("myuser", &info.room_id)
            .await
            .expect("Could not add user to room.");

        db.clear_info(&info.room_id)
            .await
            .expect("Could not clear room info");

        let users_in_room = db
            .get_users_in_room(&info.room_id)
            .await
            .expect("Could not get users in room.");

        assert_eq!(users_in_room.len(), 0);

        let room_info = db
            .get_room_info(&info.room_id)
            .await
            .expect("Could not get room info.");

        assert!(room_info.is_none());
    }
}
