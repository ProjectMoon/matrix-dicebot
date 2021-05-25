use serde::{Deserialize, Serialize};

/// RoomInfo has basic metadata about a room: its name, ID, etc.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RoomInfo {
    pub room_id: String,
    pub room_name: String,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum AccountStatus {
    /// User is not registered, which means the "account" only exists
    /// for state management in the bot. No privileged actions
    /// possible.
    NotRegistered,

    /// User account is fully registered, either via Matrix directly,
    /// or a web UI sign-up.
    Registered,

    /// Account is awaiting activation with a registration
    /// code. Account cannot do privileged actions yet.
    AwaitingActivation,
}

impl Default for AccountStatus {
    fn default() -> Self {
        AccountStatus::NotRegistered
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Default, sqlx::FromRow)]
pub struct User {
    pub username: String,
    pub password: Option<String>,
    pub active_room: Option<String>,
    pub account_status: AccountStatus,
}

impl User {
    /// Create a new unregistered skeleton marker account for a
    /// username.
    pub fn unregistered(username: &str) -> User {
        User {
            username: username.to_owned(),
            ..Default::default()
        }
    }

    pub fn verify_password(&self, raw_password: &str) -> bool {
        self.password
            .as_ref()
            .and_then(|p| argon2::verify_encoded(p, raw_password.as_bytes()).ok())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_password_passes_with_correct_password() {
        let user = User {
            password: Some(
                crate::logic::hash_password("mypassword").expect("Password hashing error!"),
            ),
            ..Default::default()
        };

        assert_eq!(user.verify_password("mypassword"), true);
    }

    #[test]
    fn verify_password_fails_with_wrong_password() {
        let user = User {
            password: Some(
                crate::logic::hash_password("mypassword").expect("Password hashing error!"),
            ),
            ..Default::default()
        };

        assert_eq!(user.verify_password("wrong-password"), false);
    }

    #[test]
    fn verify_password_fails_with_no_password() {
        let user = User {
            password: None,
            ..Default::default()
        };

        assert_eq!(user.verify_password("wrong-password"), false);
    }
}
