//! [`User`] definitions.

pub mod session;

use std::sync::LazyLock;

#[cfg(doc)]
use common::DateTime;
use common::{unit, DateTimeOf};
use derive_more::{AsRef, Display, From, FromStr, Into};
#[cfg(feature = "postgres")]
use postgres_types::{FromSql, ToSql};
use regex::Regex;
use secrecy::{zeroize::Zeroize, CloneableSecret};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use self::session::Session;

/// Platform user.
#[derive(Clone, Debug, From)]
pub struct User {
    /// ID of this [`User`]
    pub id: Id,

    /// [`Name`] of this [`User`].
    pub name: Name,

    /// [`Login`] of this [`User`].
    pub login: Login,

    /// [`PasswordHash`] of this [`User`].
    pub password_hash: PasswordHash,

    /// [`Email`] of this [`User`].
    pub email: Option<Email>,

    /// [`Phone`] of this [`User`].
    pub phone: Option<Phone>,

    /// [`DateTime`] when this [`User`] was created.
    pub created_at: CreationDateTime,

    /// [`DateTime`] when this [`User`] was deleted.
    pub deleted_at: Option<DeletionDateTime>,
}

/// ID of a [`User`].
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Display,
    Eq,
    From,
    FromStr,
    Hash,
    Into,
    PartialEq,
    Serialize,
)]
#[cfg_attr(feature = "postgres", derive(ToSql, FromSql), postgres(transparent))]
pub struct Id(Uuid);

impl Id {
    /// Creates a new random [`Id`].
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Name of a [`User`].
#[derive(AsRef, Clone, Debug, Display, Eq, PartialEq)]
#[as_ref(str, String)]
#[cfg_attr(feature = "postgres", derive(FromSql, ToSql), postgres(transparent))]
pub struct Name(String);

impl Name {
    /// Creates a new [`Name`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `name` matches the format.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Creates a new [`Name`] if the given `name` is valid.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        Self::check(&name).then_some(Self(name))
    }

    /// Checks whether the given `name` is a valid [`Name`].
    fn check(name: impl AsRef<str>) -> bool {
        let name = name.as_ref();
        name.trim() == name && !name.is_empty() && name.len() <= 512
    }
}

impl FromStr for Name {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or("invalid `Name`")
    }
}

/// Login of a [`User`].
#[derive(Clone, Debug, Display, Eq, PartialEq)]
#[cfg_attr(feature = "postgres", derive(FromSql, ToSql), postgres(transparent))]
pub struct Login(String);

impl Login {
    /// Creates a new [`Login`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `login` matches the format.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(login: impl Into<String>) -> Self {
        Self(login.into())
    }

    /// Creates a new [`Login`] if the given `login` is valid.
    #[must_use]
    pub fn new(login: impl Into<String>) -> Option<Self> {
        let login = login.into();
        Self::check(&login).then_some(Self(login))
    }

    /// Checks whether the given `login` is a valid [`Login`].
    fn check(login: impl AsRef<str>) -> bool {
        /// Regular expression checking [`Login`] invariants:
        /// - Must not be empty;
        /// - Must not start/end with whitespace;
        /// - Must not contain consecutive whitespace;
        /// - Must not contain control characters;
        /// - Must not contain special characters;
        /// - Must be between 1 and 20 characters long.
        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^\S[\p{L}\p{N}]{0,98}\S$").expect("valid regex")
        });

        REGEX.is_match(login.as_ref())
    }
}

impl FromStr for Login {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or("invalid `Login`")
    }
}

/// Password of a [`User`].
#[derive(Clone, Debug, Display, Eq, From, PartialEq)]
#[from(&str, String)]
pub struct Password(String);

impl Password {
    /// Creates a new [`Password`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `password` matches the format.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(password: impl Into<String>) -> Self {
        Self(password.into())
    }

    /// Creates a new [`Password`] if the given `password` is valid.
    #[must_use]
    pub fn new(password: impl Into<String>) -> Option<Self> {
        let password = password.into();
        Self::check(&password).then_some(Self(password))
    }

    /// Checks whether the given `password` is a valid [`Password`].
    fn check(password: impl AsRef<str>) -> bool {
        let password = password.as_ref();
        password.len() > 1 && password.len() <= 128
    }
}

impl FromStr for Password {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or("invalid `Password`")
    }
}

impl CloneableSecret for Password {}
impl Zeroize for Password {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

/// Password hash of a [`User`].
#[derive(Clone, Debug, Display, Eq, PartialEq)]
#[cfg_attr(feature = "postgres", derive(FromSql, ToSql), postgres(transparent))]
pub struct PasswordHash(String);

impl PasswordHash {
    /// Creates a new [`PasswordHash`] from the given [`Password`].
    #[must_use]
    pub fn new(password: &Password) -> Self {
        // TODO: Use `argon2` or any other secure hashing algorithm.
        Self(password.to_string())
    }
}

/// Email address of a [`User`].
#[derive(AsRef, Clone, Debug, Display, Eq, PartialEq)]
#[as_ref(str, String)]
#[cfg_attr(feature = "postgres", derive(FromSql, ToSql), postgres(transparent))]
pub struct Email(String);

impl Email {
    /// Creates a new [`Email`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `address` matches the format.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(address: impl Into<String>) -> Self {
        Self(address.into())
    }

    /// Creates a new [`Email`] if the given `address` is valid.
    #[must_use]
    pub fn new(address: impl Into<String>) -> Option<Self> {
        let address = address.into();
        Self::check(&address).then_some(Self(address))
    }

    /// Checks whether the given `address` is a valid [`Email`].
    fn check(address: impl AsRef<str>) -> bool {
        /// Regular expression checking [`Email`] format.
        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(
                "^([^\\x00-\\x20\\x22\\x28\\x29\\x2c\\x2e\\x3a-\
                     \\x3c\\x3e\\x40\\x5b-\\x5d\\x7f-\\xff]+\
                  |\\x22([^\\x0d\\x22\\x5c\\x80-\\xff]\
                  |\\x5c[\\x00-\\x7f])*\\x22)\
                  (\\x2e([^\\x00-\\x20\\x22\\x28\\x29\\x2c\\x2e\\x3a-\
                           \\x3c\\x3e\\x40\\x5b-\\x5d\\x7f-\\xff]+\
                        |\\x22([^\\x0d\\x22\\x5c\\x80-\\xff]\
                        |\\x5c[\\x00-\\x7f])*\\x22))*\\x40\
                  ([^\\x00-\\x20\\x22\\x28\\x29\\x2c\\x2e\\x3a-\
                     \\x3c\\x3e\\x40\\x5b-\\x5d\\x7f-\\xff]+\
                  |\\x5b([^\\x0d\\x5b-\\x5d\\x80-\\xff]\
                        |\\x5c[\\x00-\\x7f])*\\x5d)\
                  (\\x2e([^\\x00-\\x20\\x22\\x28\\x29\\x2c\\x2e\\x3a-\
                           \\x3c\\x3e\\x40\\x5b-\\x5d\\x7f-\\xff]+\
                        |\\x5b([^\\x0d\\x5b-\\x5d\\x80-\\xff]\
                        |\\x5c[\\x00-\\x7f])*\\x5d))*$",
            )
            .expect("valid regex")
        });

        REGEX.is_match(address.as_ref())
    }
}

impl FromStr for Email {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or("invalid `Email`")
    }
}

/// Phone number of a [`User`].
#[derive(AsRef, Clone, Debug, Display, Eq, PartialEq)]
#[as_ref(str, String)]
#[cfg_attr(feature = "postgres", derive(FromSql, ToSql), postgres(transparent))]
pub struct Phone(String);

impl Phone {
    /// Creates a new [`Phone`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `number` matches the format.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(number: impl Into<String>) -> Self {
        Self(number.into())
    }

    /// Creates a new [`Phone`] if the given `address` is valid.
    #[must_use]
    pub fn new(number: impl Into<String>) -> Option<Self> {
        let number = number.into();
        Self::check(&number).then_some(Self(number))
    }

    /// Checks whether the given `number` is a valid [`Phone`].
    fn check(number: impl AsRef<str>) -> bool {
        /// Regular expression checking [`Phone`] format.
        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^([+]?\d{1,2}[-\s]?|)\d{3}[-\s]?\d{3}[-\s]?\d{4}$")
                .expect("valid regex")
        });

        REGEX.is_match(number.as_ref())
    }
}

impl FromStr for Phone {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or("invalid `Phone`")
    }
}

/// [`DateTime`] when a [`User`] was created.
pub type CreationDateTime = DateTimeOf<(User, unit::Creation)>;

/// [`DateTime`] when a [`User`] was deleted.
pub type DeletionDateTime = DateTimeOf<(User, unit::Deletion)>;
