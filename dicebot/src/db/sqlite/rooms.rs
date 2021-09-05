use super::Database;
use crate::db::{errors::DataError, Rooms};
use async_trait::async_trait;
use sqlx::SqlitePool;
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
}

#[cfg(test)]
mod tests {
    use crate::db::sqlite::Database;
    use crate::db::Rooms;
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
    async fn should_process_test() {
        with_db(|db| async move {
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
        })
        .await;
    }
}
