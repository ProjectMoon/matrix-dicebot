use crate::db::errors::DataError;
use crate::db::schema::convert_u64;
use crate::models::RoomInfo;
use byteorder::BigEndian;
use log::{debug, error, log_enabled};
use sled::transaction::TransactionalTree;
use sled::Transactional;
use sled::{CompareAndSwapError, Tree};
use std::collections::HashSet;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::task::JoinHandle;
use zerocopy::byteorder::U64;
use zerocopy::AsBytes;

#[derive(Clone)]
pub struct Rooms {
    /// Room ID -> RoomInfo struct (single entries).
    /// Key is just room ID as bytes.
    pub(in crate::db) roomid_roominfo: Tree,

    /// Room ID -> list of usernames in room.
    pub(in crate::db) roomid_usernames: Tree,

    /// Username -> list of room IDs user is in.
    pub(in crate::db) username_roomids: Tree,

    /// Room ID(str) 0xff event ID(str) -> timestamp. Records event
    /// IDs that we have received, so we do not process twice.
    pub(in crate::db) roomeventid_timestamp: Tree,

    /// Room ID(str) 0xff timestamp(u64) -> event ID. Records event
    /// IDs with timestamp as the primary key instead. Exists to allow
    /// easy scanning of old roomeventid_timestamp records for
    /// removal. Be careful with u64, it can have 0xff and 0xfe bytes.
    /// A simple split on 0xff will not work with this key. Instead,
    /// it is meant to be split on the first 0xff byte only, and
    /// separated into room ID and timestamp.
    pub(in crate::db) roomtimestamp_eventid: Tree,
}

/// An enum that can hold either a regular sled Tree, or a
/// Transactional tree.
#[derive(Clone, Copy)]
enum TxableTree<'a> {
    Tree(&'a Tree),
    Tx(&'a TransactionalTree),
}

impl<'a> Into<TxableTree<'a>> for &'a Tree {
    fn into(self) -> TxableTree<'a> {
        TxableTree::Tree(self)
    }
}

impl<'a> Into<TxableTree<'a>> for &'a TransactionalTree {
    fn into(self) -> TxableTree<'a> {
        TxableTree::Tx(self)
    }
}

/// A set of functions that can be used with a sled::Tree that stores
/// HashSets as its values. Atomicity is partially handled. If the
/// Tree is a transactional tree, operations will be atomic.
/// Otherwise, there is a potential non-atomic step.
mod hashset_tree {
    use super::*;

    fn insert_set<'a, T: Into<TxableTree<'a>>>(
        tree: T,
        key: &[u8],
        set: HashSet<String>,
    ) -> Result<(), DataError> {
        let serialized = bincode::serialize(&set)?;
        match tree.into() {
            TxableTree::Tree(tree) => tree.insert(key, serialized)?,
            TxableTree::Tx(tx) => tx.insert(key, serialized)?,
        };
        Ok(())
    }

    pub(super) fn get_set<'a, T: Into<TxableTree<'a>>>(
        tree: T,
        key: &[u8],
    ) -> Result<HashSet<String>, DataError> {
        let set: HashSet<String> = match tree.into() {
            TxableTree::Tree(tree) => tree.get(key)?,
            TxableTree::Tx(tx) => tx.get(key)?,
        }
        .map(|bytes| bincode::deserialize::<HashSet<String>>(&bytes))
        .unwrap_or(Ok(HashSet::new()))?;

        Ok(set)
    }

    pub(super) fn remove_from_set<'a, T: Into<TxableTree<'a>> + Copy>(
        tree: T,
        key: &[u8],
        value_to_remove: &str,
    ) -> Result<(), DataError> {
        let mut set = get_set(tree, key)?;
        set.remove(value_to_remove);
        insert_set(tree, key, set)?;
        Ok(())
    }

    pub(super) fn add_to_set<'a, T: Into<TxableTree<'a>> + Copy>(
        tree: T,
        key: &[u8],
        value_to_add: String,
    ) -> Result<(), DataError> {
        let mut set = get_set(tree, key)?;
        set.insert(value_to_add);
        insert_set(tree, key, set)?;
        Ok(())
    }
}

/// Functions that specifically relate to the "timestamp index" tree,
/// which is stored on the Rooms instance as a tree called
/// roomtimestamp_eventid. Tightly coupled to the event watcher in the
/// Rooms impl, and only factored out for unit testing.
mod timestamp_index {
    use super::*;

    /// Insert an entry from the main roomeventid_timestamp Tree into
    /// the timestamp index. Keys in this Tree are stored as room ID
    /// 0xff timestamp, with the value being a hashset of event IDs
    /// received at the time. The parameters come from an insert to
    /// that Tree, where the key is room ID 0xff event ID, and the
    /// value is the timestamp.
    pub(super) fn insert(
        roomtimestamp_eventid: &Tree,
        key: &[u8],
        timestamp_bytes: &[u8],
    ) -> Result<(), DataError> {
        let parts: Vec<&[u8]> = key.split(|&b| b == 0xff).collect();
        if let [room_id, event_id] = parts[..] {
            let mut ts_key = room_id.to_vec();
            ts_key.push(0xff);
            ts_key.extend_from_slice(&timestamp_bytes);
            log_index_record(room_id, event_id, &timestamp_bytes);

            let event_id = str::from_utf8(event_id)?;
            hashset_tree::add_to_set(roomtimestamp_eventid, &ts_key, event_id.to_owned())?;
            Ok(())
        } else {
            Err(DataError::InvalidValue)
        }
    }

    /// Log a debug message.
    fn log_index_record(room_id: &[u8], event_id: &[u8], timestamp: &[u8]) {
        if log_enabled!(log::Level::Debug) {
            debug!(
                "Recording event {} | {} received at {} in timestamp index.",
                str::from_utf8(room_id).unwrap_or("[invalid room id]"),
                str::from_utf8(event_id).unwrap_or("[invalid event id]"),
                convert_u64(timestamp).unwrap_or(0)
            );
        }
    }
}

impl Rooms {
    pub(in crate::db) fn new(db: &sled::Db) -> Result<Rooms, sled::Error> {
        Ok(Rooms {
            roomid_roominfo: db.open_tree("roomid_roominfo")?,
            roomid_usernames: db.open_tree("roomid_usernames")?,
            username_roomids: db.open_tree("username_roomids")?,
            roomeventid_timestamp: db.open_tree("roomeventid_timestamp")?,
            roomtimestamp_eventid: db.open_tree("roomtimestamp_eventid")?,
        })
    }

    /// Start an event subscriber that listens for inserts made by the
    /// `should_process` function. This event handler duplicates the
    /// entry by timestamp instead of event ID.
    pub(in crate::db) fn start_handler(&self) -> JoinHandle<()> {
        //Clone due to lifetime requirements.
        let roomeventid_timestamp = self.roomeventid_timestamp.clone();
        let roomtimestamp_eventid = self.roomtimestamp_eventid.clone();

        tokio::spawn(async move {
            let mut subscriber = roomeventid_timestamp.watch_prefix(b"");

            // TODO make this handler receive kill messages somehow so
            // we can unit test it and gracefully shut it down.
            while let Some(event) = (&mut subscriber).await {
                if let sled::Event::Insert { key, value } = event {
                    match timestamp_index::insert(&roomtimestamp_eventid, &key, &value) {
                        Err(e) => {
                            error!("Unable to update the timestamp index: {}", e);
                        }
                        _ => (),
                    }
                }
            }
        })
    }

    /// Determine if an event in a room should be processed. The event
    /// is atomically recorded and true returned if the database has
    /// not seen tis event yet. If the event already exists in the
    /// database, the function returns false. Events are recorded by
    /// this function by inserting the (system-local) timestamp in
    /// epoch seconds.
    pub fn should_process(&self, room_id: &str, event_id: &str) -> Result<bool, DataError> {
        let mut key = room_id.as_bytes().to_vec();
        key.push(0xff);
        key.extend_from_slice(event_id.as_bytes());

        let timestamp: U64<BigEndian> = U64::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Clock has gone backwards")
                .as_secs(),
        );

        match self.roomeventid_timestamp.compare_and_swap(
            key,
            None as Option<&[u8]>,
            Some(timestamp.as_bytes()),
        )? {
            Ok(()) => Ok(true),
            Err(CompareAndSwapError { .. }) => Ok(false),
        }
    }

    pub fn insert_room_info(&self, info: &RoomInfo) -> Result<(), DataError> {
        let key = info.room_id.as_bytes();
        let serialized = bincode::serialize(&info)?;
        self.roomid_roominfo.insert(key, serialized)?;
        Ok(())
    }

    pub fn get_room_info(&self, room_id: &str) -> Result<Option<RoomInfo>, DataError> {
        let key = room_id.as_bytes();

        let room_info: Option<RoomInfo> = self
            .roomid_roominfo
            .get(key)?
            .map(|bytes| bincode::deserialize(&bytes))
            .transpose()?;

        Ok(room_info)
    }

    pub fn get_rooms_for_user(&self, username: &str) -> Result<HashSet<String>, DataError> {
        hashset_tree::get_set(&self.username_roomids, username.as_bytes())
    }

    pub fn get_users_in_room(&self, room_id: &str) -> Result<HashSet<String>, DataError> {
        hashset_tree::get_set(&self.roomid_usernames, room_id.as_bytes())
    }

    pub fn add_user_to_room(&self, username: &str, room_id: &str) -> Result<(), DataError> {
        debug!("Adding user {} to room {}", username, room_id);
        (&self.username_roomids, &self.roomid_usernames).transaction(
            |(tx_username_rooms, tx_room_usernames)| {
                let username_key = &username.as_bytes();
                hashset_tree::add_to_set(tx_username_rooms, username_key, room_id.to_owned())?;

                let roomid_key = &room_id.as_bytes();
                hashset_tree::add_to_set(tx_room_usernames, roomid_key, username.to_owned())?;

                Ok(())
            },
        )?;

        Ok(())
    }

    pub fn remove_user_from_room(&self, username: &str, room_id: &str) -> Result<(), DataError> {
        debug!("Removing user {} from room {}", username, room_id);
        (&self.username_roomids, &self.roomid_usernames).transaction(
            |(tx_username_rooms, tx_room_usernames)| {
                let username_key = &username.as_bytes();
                hashset_tree::remove_from_set(tx_username_rooms, username_key, room_id)?;

                let roomid_key = &room_id.as_bytes();
                hashset_tree::remove_from_set(tx_room_usernames, roomid_key, username)?;

                Ok(())
            },
        )?;

        Ok(())
    }

    pub fn clear_info(&self, room_id: &str) -> Result<(), DataError> {
        debug!("Clearing all information for room {}", room_id);
        (&self.username_roomids, &self.roomid_usernames).transaction(
            |(tx_username_roomids, tx_roomid_usernames)| {
                let roomid_key = room_id.as_bytes();
                let users_in_room = hashset_tree::get_set(tx_roomid_usernames, roomid_key)?;

                //Remove the room ID from every user's room ID list.
                for username in users_in_room {
                    hashset_tree::remove_from_set(
                        tx_username_roomids,
                        username.as_bytes(),
                        room_id,
                    )?;
                }

                //Remove this room entry for the room ID -> username tree.
                tx_roomid_usernames.remove(roomid_key)?;

                //TODO: delete roominfo struct from room info tree.
                Ok(())
            },
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sled::Config;

    fn create_test_instance() -> Rooms {
        let config = Config::new().temporary(true);
        let db = config.open().unwrap();
        Rooms::new(&db).unwrap()
    }

    #[test]
    fn add_user_to_room() {
        let rooms = create_test_instance();

        rooms
            .add_user_to_room("testuser", "myroom")
            .expect("Could not add user to room");

        let users_in_room = rooms
            .get_users_in_room("myroom")
            .expect("Could not retrieve users in room");

        let rooms_for_user = rooms
            .get_rooms_for_user("testuser")
            .expect("Could not get rooms for user");

        let expected_users_in_room: HashSet<String> =
            vec![String::from("testuser")].into_iter().collect();

        let expected_rooms_for_user: HashSet<String> =
            vec![String::from("myroom")].into_iter().collect();

        assert_eq!(expected_users_in_room, users_in_room);
        assert_eq!(expected_rooms_for_user, rooms_for_user);
    }

    #[test]
    fn remove_user_from_room() {
        let rooms = create_test_instance();

        rooms
            .add_user_to_room("testuser", "myroom")
            .expect("Could not add user to room");

        rooms
            .remove_user_from_room("testuser", "myroom")
            .expect("Could not remove user from room");

        let users_in_room = rooms
            .get_users_in_room("myroom")
            .expect("Could not retrieve users in room");

        let rooms_for_user = rooms
            .get_rooms_for_user("testuser")
            .expect("Could not get rooms for user");

        assert_eq!(HashSet::new(), users_in_room);
        assert_eq!(HashSet::new(), rooms_for_user);
    }

    #[test]
    fn insert_room_info_works() {
        let rooms = create_test_instance();

        let info = RoomInfo {
            room_id: matrix_sdk::identifiers::room_id!("!fakeroom:example.com")
                .as_str()
                .to_owned(),
            room_name: "fake room name".to_owned(),
        };

        rooms
            .insert_room_info(&info)
            .expect("Could insert room info");

        let found_info = rooms
            .get_room_info("!fakeroom:example.com")
            .expect("Error loading room info");

        assert!(found_info.is_some());
        assert_eq!(info, found_info.unwrap());
    }

    #[test]
    fn get_room_info_none_when_room_does_not_exist() {
        let rooms = create_test_instance();

        let found_info = rooms
            .get_room_info("!fakeroom:example.com")
            .expect("Error loading room info");

        assert!(found_info.is_none());
    }

    #[test]
    fn clear_info_modifies_removes_requested_room() {
        let rooms = create_test_instance();

        rooms
            .add_user_to_room("testuser", "myroom1")
            .expect("Could not add user to room1");

        rooms
            .add_user_to_room("testuser", "myroom2")
            .expect("Could not add user to room2");

        rooms
            .clear_info("myroom1")
            .expect("Could not clear room info");

        let users_in_room1 = rooms
            .get_users_in_room("myroom1")
            .expect("Could not retrieve users in room1");

        let users_in_room2 = rooms
            .get_users_in_room("myroom2")
            .expect("Could not retrieve users in room2");

        let rooms_for_user = rooms
            .get_rooms_for_user("testuser")
            .expect("Could not get rooms for user");

        let expected_users_in_room2: HashSet<String> =
            vec![String::from("testuser")].into_iter().collect();

        let expected_rooms_for_user: HashSet<String> =
            vec![String::from("myroom2")].into_iter().collect();

        assert_eq!(HashSet::new(), users_in_room1);
        assert_eq!(expected_users_in_room2, users_in_room2);
        assert_eq!(expected_rooms_for_user, rooms_for_user);
    }

    #[test]
    fn insert_to_timestamp_index() {
        let rooms = create_test_instance();

        // Insertion into timestamp index based on data that would go
        // into main room x eventID -> timestamp tree.
        let mut key = b"myroom".to_vec();
        key.push(0xff);
        key.extend_from_slice(b"myeventid");

        let timestamp: U64<BigEndian> = U64::new(1000);

        let result = timestamp_index::insert(
            &rooms.roomtimestamp_eventid,
            key.as_bytes(),
            timestamp.as_bytes(),
        );

        assert!(result.is_ok());

        // Retrieval of data from the timestamp index tree.
        let mut ts_key = b"myroom".to_vec();
        ts_key.push(0xff);
        ts_key.extend_from_slice(timestamp.as_bytes());

        let expected_events: HashSet<String> =
            vec![String::from("myeventid")].into_iter().collect();

        let event_ids = hashset_tree::get_set(&rooms.roomtimestamp_eventid, &ts_key)
            .expect("Could not get set out of Tree");
        assert_eq!(expected_events, event_ids);
    }
}
