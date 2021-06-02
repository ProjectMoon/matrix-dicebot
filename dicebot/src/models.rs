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
    /// Account is not registered, which means a transient "account"
    /// with limited information exists only for the duration of the
    /// command request.
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Account {
    /// A registered user account, stored in the database.
    Registered(User),

    /// A transient account. Not stored in the database. Represents a
    /// user in a public channel that has not registered directly with
    /// the bot yet.
    Transient(TransientUser),
}

impl Account {
    /// Whether or not this account is a registered user account.
    pub fn is_registered(&self) -> bool {
        matches!(self, Self::Registered(_))
    }

    /// Gets the account status. For registered users, this is their
    /// actual account status (fully registered or awaiting
    /// activation). For transient users, this is
    /// AccountStatus::NotRegistered.
    pub fn account_status(&self) -> AccountStatus {
        match self {
            Self::Registered(user) => user.account_status,
            Self::Transient(_) => AccountStatus::NotRegistered,
        }
    }

    /// Consume self into an Option<User> instance, which will be Some
    /// if this account has a registered user, and None otherwise.
    pub fn registered_user(&self) -> Option<&User> {
        match self {
            Self::Registered(ref user) => Some(user),
            _ => None,
        }
    }

    /// Consume self into an Option<TransientUser> instance, which
    /// will be Some if this account has a non-registered user, and
    /// None otherwise.
    pub fn transient_user(self) -> Option<TransientUser> {
        match self {
            Self::Transient(user) => Some(user),
            _ => None,
        }
    }
}

impl Default for Account {
    fn default() -> Self {
        Account::Transient(TransientUser {
            username: "".to_string(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransientUser {
    pub username: String,
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
