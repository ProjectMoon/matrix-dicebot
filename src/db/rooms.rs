use crate::db::errors::DataError;
use sled::transaction::TransactionalTree;
use sled::Transactional;
use sled::Tree;
use std::collections::HashSet;
use std::str;

#[derive(Clone)]
pub struct Rooms {
    /// Room ID -> RoomInfo struct (single entries)
    pub(in crate::db) roomid_roominfo: Tree,

    /// Room ID -> list of usernames in room.
    pub(in crate::db) roomid_usernames: Tree,

    /// Username -> list of room IDs user is in.
    pub(in crate::db) username_roomids: Tree,
}

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

fn get_set<'a, T: Into<TxableTree<'a>>>(tree: T, key: &[u8]) -> Result<HashSet<String>, DataError> {
    let set: HashSet<String> = match tree.into() {
        TxableTree::Tree(tree) => tree.get(key)?,
        TxableTree::Tx(tx) => tx.get(key)?,
    }
    .map(|bytes| bincode::deserialize::<HashSet<String>>(&bytes))
    .unwrap_or(Ok(HashSet::new()))?;

    Ok(set)
}

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

impl Rooms {
    pub(in crate::db) fn new(db: &sled::Db) -> Result<Rooms, sled::Error> {
        Ok(Rooms {
            roomid_roominfo: db.open_tree("roomid_roominfo")?,
            roomid_usernames: db.open_tree("roomid_usernames")?,
            username_roomids: db.open_tree("username_roomids")?,
        })
    }

    pub fn get_rooms_for_user(&self, username: &str) -> Result<HashSet<String>, DataError> {
        get_set(&self.username_roomids, username.as_bytes())
    }

    pub fn get_users_in_room(&self, room_id: &str) -> Result<HashSet<String>, DataError> {
        get_set(&self.roomid_usernames, room_id.as_bytes())
    }

    pub fn add_user_to_room(&self, username: &str, room_id: &str) -> Result<(), DataError> {
        (&self.username_roomids, &self.roomid_usernames).transaction(
            |(tx_username_rooms, tx_room_usernames)| {
                let username_key = &username.as_bytes();
                let mut user_to_rooms = get_set(tx_username_rooms, username_key)?;
                user_to_rooms.insert(room_id.to_string());
                insert_set(tx_username_rooms, username_key, user_to_rooms)?;

                let roomid_key = &room_id.as_bytes();
                let mut room_to_users = get_set(tx_room_usernames, roomid_key)?;
                room_to_users.insert(username.to_string());
                insert_set(tx_room_usernames, roomid_key, room_to_users)?;

                Ok(())
            },
        )?;

        Ok(())
    }

    pub fn remove_user_from_room(&self, username: &str, room_id: &str) -> Result<(), DataError> {
        (&self.username_roomids, &self.roomid_usernames).transaction(
            |(tx_username_rooms, tx_room_usernames)| {
                let username_key = &username.as_bytes();
                let mut user_to_rooms = get_set(tx_username_rooms, username_key)?;
                user_to_rooms.remove(room_id);
                insert_set(tx_username_rooms, username_key, user_to_rooms)?;

                let roomid_key = &room_id.as_bytes();
                let mut room_to_users = get_set(tx_room_usernames, roomid_key)?;
                room_to_users.remove(username);
                insert_set(tx_room_usernames, roomid_key, room_to_users)?;

                Ok(())
            },
        )?;

        Ok(())
    }

    pub fn clear_info(&self, _room_id: &str) -> Result<(), DataError> {
        //TODO implement me
        //when bot leaves a room, it must, atomically:
        // - delete roominfo struct from room info tree.
        // - load list of users it knows about in room.
        // - remove room id from every user's list. (cannot reuse existing fn because atomicity)
        // - delete list of users in room from tree.
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
}
