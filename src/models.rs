use serde::{Deserialize, Serialize};

/// RoomInfo has basic metadata about a room: its name, ID, etc.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RoomInfo {
    pub room_id: String,
    pub room_name: String,
}

#[derive(Eq, PartialEq, Debug)]
pub struct User {
    pub username: String,
    pub password: String,
}

impl User {
    pub fn verify_password(&self, raw_password: &str) -> bool {
        argon2::verify_encoded(&self.password, raw_password.as_bytes()).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_password_passes_with_correct_password() {
        let user = User {
            username: "myuser".to_string(),
            password: crate::logic::hash_password("mypassword").expect("Password hashing error!"),
        };

        assert_eq!(user.verify_password("mypassword"), true);
    }

    #[test]
    fn verify_password_fails_with_wrong_password() {
        let user = User {
            username: "myuser".to_string(),
            password: crate::logic::hash_password("mypassword").expect("Password hashing error!"),
        };

        assert_eq!(user.verify_password("wrong-password"), false);
    }
}
