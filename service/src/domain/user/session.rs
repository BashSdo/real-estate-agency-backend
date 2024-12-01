//! [`Session`] definitions.

#[cfg(doc)]
use common::DateTime;
use common::DateTimeOf;
use derive_more::{AsRef, Display, FromStr};
use serde::{Deserialize, Serialize};

#[cfg(doc)]
use crate::domain::User;
use crate::domain::{contract::Expiration, user};

/// User session.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Session {
    /// ID of the [`User`] this [`Session`] belongs to.
    pub user_id: user::Id,

    /// [`DateTime`] when this [`Session`] expires.
    #[serde(rename = "exp", with = "common::datetime::serde::unix_timestamp")]
    pub expires_at: ExpirationDateTime,
}

/// Access token of a [`Session`].
#[derive(AsRef, Clone, Debug, Display, FromStr)]
pub struct Token(String);

impl Token {
    /// Creates a new [`Token`] without checking its contents.
    ///
    /// # Safety
    ///
    /// The provided `token` must be a valid [`Token`] representation.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub const unsafe fn new_unchecked(token: String) -> Self {
        Self(token)
    }
}

/// [`DateTime`] of a [`Session`] expiration.
pub type ExpirationDateTime = DateTimeOf<(Session, Expiration)>;
