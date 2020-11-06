use crate::db::errors::DataError;
use crate::db::schema::convert_i32;
use byteorder::LittleEndian;
use sled::transaction::{abort, TransactionalTree};
use sled::Transactional;
use sled::Tree;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::From;
use std::str;
use zerocopy::byteorder::I32;
use zerocopy::AsBytes;

#[derive(Clone)]
pub struct Rooms {
    /// Room ID -> RoomInfo struct (single entries)
    pub(in crate::db) roomid_roominfo: Tree,

    /// Room ID -> list of usernames in room.
    pub(in crate::db) roomid_usernames: Tree,

    /// Username -> list of room IDs user is in.
    pub(in crate::db) username_roomids: Tree,
}

// /// Request soemthing by a username and room ID.
// pub struct UserAndRoom<'a>(pub &'a str, pub &'a str);

// fn to_vec(value: &UserAndRoom<'_>) -> Vec<u8> {
//     let mut bytes = vec![];
//     bytes.extend_from_slice(value.0.as_bytes());
//     bytes.push(0xfe);
//     bytes.extend_from_slice(value.1.as_bytes());
//     bytes
// }

// impl From<UserAndRoom<'_>> for Vec<u8> {
//     fn from(value: UserAndRoom) -> Vec<u8> {
//         to_vec(&value)
//     }
// }

// impl From<&UserAndRoom<'_>> for Vec<u8> {
//     fn from(value: &UserAndRoom) -> Vec<u8> {
//         to_vec(value)
//     }
// }

impl Rooms {
    pub(in crate::db) fn new(db: &sled::Db) -> Result<Rooms, sled::Error> {
        Ok(Rooms {
            roomid_roominfo: db.open_tree("roomid_roominfo")?,
            roomid_usernames: db.open_tree("roomid_usernames")?,
            username_roomids: db.open_tree("username_roomids")?,
        })
    }

    pub fn add_user_to_room(&self, username: &str, room_id: &str) -> Result<(), DataError> {
        //in txn:
        //get or create list of users in room
        //get or create list of rooms user is in
        //deserialize/create set and add username to set for roomid
        //deserialize/create set and add roomid to set for username
        //store both again
        let user_to_rooms: HashSet<String> = self
            .username_roomids
            .get(username.as_bytes())?
            .map(|bytes| bincode::deserialize::<HashSet<String>>(&bytes))
            .unwrap_or(Ok(HashSet::new()))?;

        let room_to_users: HashSet<String> = self
            .roomid_usernames
            .get(room_id.as_bytes())?
            .map(|bytes| bincode::deserialize::<HashSet<String>>(&bytes))
            .unwrap_or(Ok(HashSet::new()))?;
        Ok(())
    }

    pub fn remove_user_from_room(&self, username: &str, room_id: &str) -> Result<(), DataError> {
        Ok(())
    }
}
