//! [`Contract`] definitions.

pub mod employment;
pub mod management_for_rent;
pub mod management_for_sale;
pub mod rent;
pub mod sale;

use common::{define_kind, unit, DateTime, DateTimeOf};
use derive_more::{AsRef, Display, From, FromStr, Into};
#[cfg(feature = "postgres")]
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::realty;
#[cfg(doc)]
use crate::domain::Realty;

pub use self::{
    employment::Employment, management_for_rent::ManagementForRent,
    management_for_sale::ManagementForSale, rent::Rent, sale::Sale,
};

/// [`Realty`] contract.
#[derive(Clone, Debug, From)]
pub enum Contract {
    #[doc(hidden)]
    Rent(Rent),
    #[doc(hidden)]
    Sale(Sale),
    #[doc(hidden)]
    ManagementForRent(ManagementForRent),
    #[doc(hidden)]
    ManagementForSale(ManagementForSale),
    #[doc(hidden)]
    Employment(Employment),
}

impl Contract {
    /// Returns ID of this [`Contract`].
    #[must_use]
    pub fn id(&self) -> Id {
        match self {
            Self::Rent(c) => c.id,
            Self::Sale(c) => c.id,
            Self::ManagementForRent(c) => c.id,
            Self::ManagementForSale(c) => c.id,
            Self::Employment(c) => c.id,
        }
    }

    /// Returns [`Kind`] of this [`Contract`].
    #[must_use]
    pub fn kind(&self) -> Kind {
        match self {
            Self::Rent(_) => Kind::Rent,
            Self::Sale(_) => Kind::Sale,
            Self::ManagementForRent(_) => Kind::ManagementForRent,
            Self::ManagementForSale(_) => Kind::ManagementForSale,
            Self::Employment(_) => Kind::Employment,
        }
    }

    /// Returns [`Status`] of this [`Contract`].
    #[must_use]
    pub fn status(&self) -> Status {
        use Status as S;

        if self.terminated_at().is_some() {
            return S::Terminated;
        }

        if let Some(at) = self.expires_at() {
            let now = DateTime::now().coerce();
            if now > at {
                return S::Completed;
            }
        }

        S::Active
    }

    /// Returns [`Name`] of this [`Contract`].
    #[must_use]
    pub fn name(&self) -> &Name {
        match self {
            Self::Rent(c) => &c.name,
            Self::Sale(c) => &c.name,
            Self::ManagementForRent(c) => &c.name,
            Self::ManagementForSale(c) => &c.name,
            Self::Employment(c) => &c.name,
        }
    }

    /// Returns [`Description`] of this [`Contract`].
    #[must_use]
    pub fn description(&self) -> &Description {
        match self {
            Self::Rent(c) => &c.description,
            Self::Sale(c) => &c.description,
            Self::ManagementForRent(c) => &c.description,
            Self::ManagementForSale(c) => &c.description,
            Self::Employment(c) => &c.description,
        }
    }

    /// Returns ID of the [`Realty`] related to this [`Contract`].
    ///
    /// [`None`] is returned in case of this [`Contract`] is not related to any
    /// [`Realty`].
    #[must_use]
    pub fn realty_id(&self) -> Option<realty::Id> {
        match self {
            Self::Rent(c) => Some(c.realty_id),
            Self::Sale(c) => Some(c.realty_id),
            Self::ManagementForRent(c) => Some(c.realty_id),
            Self::ManagementForSale(c) => Some(c.realty_id),
            Self::Employment(_) => None,
        }
    }

    /// Returns whether this [`Contract`] is placed.
    ///
    /// [`None`] is returned in case of placing is not supported for this
    /// [`Contract`].
    #[must_use]
    pub fn is_placed(&self) -> Option<bool> {
        match self {
            Self::ManagementForRent(c) => Some(c.is_placed),
            Self::ManagementForSale(c) => Some(c.is_placed),
            Self::Employment(_) | Self::Rent(_) | Self::Sale(_) => None,
        }
    }

    /// Returns whether this [`Contract`] is placed.
    ///
    /// [`None`] is returned in case of placing is not supported for this
    /// [`Contract`].
    #[must_use]
    pub fn is_placed_mut(&mut self) -> Option<&mut bool> {
        match self {
            Self::ManagementForRent(c) => Some(&mut c.is_placed),
            Self::ManagementForSale(c) => Some(&mut c.is_placed),
            Self::Employment(_) | Self::Rent(_) | Self::Sale(_) => None,
        }
    }

    /// Returns [`DateTime`] when this [`Contract`] was created.
    #[must_use]
    pub fn created_at(&self) -> CreationDateTime {
        match self {
            Self::Rent(c) => c.created_at,
            Self::Sale(c) => c.created_at,
            Self::ManagementForRent(c) => c.created_at,
            Self::ManagementForSale(c) => c.created_at,
            Self::Employment(c) => c.created_at,
        }
    }

    /// Returns [`DateTime`] when this [`Contract`] expires.
    ///
    /// [`None`] means that this [`Contract`] is valid indefinitely.
    #[must_use]
    pub fn expires_at(&self) -> Option<ExpirationDateTime> {
        match self {
            Self::Rent(c) => c.expires_at,
            Self::Sale(c) => c.expires_at,
            Self::ManagementForRent(c) => c.expires_at,
            Self::ManagementForSale(c) => c.expires_at,
            Self::Employment(c) => c.expires_at,
        }
    }

    /// Returns [`DateTime`] when this [`Contract`] was terminated, if it was.
    #[must_use]
    pub fn terminated_at(&self) -> Option<TerminationDateTime> {
        match self {
            Self::Rent(c) => c.terminated_at,
            Self::Sale(c) => c.terminated_at,
            Self::ManagementForRent(c) => c.terminated_at,
            Self::ManagementForSale(c) => c.terminated_at,
            Self::Employment(c) => c.terminated_at,
        }
    }

    /// Returns [`DateTime`] when this [`Contract`] was terminated, if it was.
    #[must_use]
    pub fn terminated_at_mut(&mut self) -> &mut Option<TerminationDateTime> {
        match self {
            Self::Rent(c) => &mut c.terminated_at,
            Self::Sale(c) => &mut c.terminated_at,
            Self::ManagementForRent(c) => &mut c.terminated_at,
            Self::ManagementForSale(c) => &mut c.terminated_at,
            Self::Employment(c) => &mut c.terminated_at,
        }
    }

    /// Returns whether this [`Contract`] is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        match self {
            Self::Rent(c) => c.is_active(),
            Self::Sale(c) => c.is_active(),
            Self::ManagementForRent(c) => c.is_active(),
            Self::ManagementForSale(c) => c.is_active(),
            Self::Employment(c) => c.is_active(),
        }
    }
}

/// ID of a [`Contract`].
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

/// Name of a [`Contract`].
#[derive(AsRef, Clone, Debug, Display, Eq, PartialEq)]
#[as_ref(str, String)]
#[cfg_attr(feature = "postgres", derive(FromSql, ToSql), postgres(transparent))]
pub struct Name(String);

impl Name {
    /// Creates a new [`Name`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `name` is not empty.
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

/// Description of a [`Contract`].
#[derive(Clone, Debug, Display, Eq, PartialEq)]
#[cfg_attr(feature = "postgres", derive(FromSql, ToSql), postgres(transparent))]
pub struct Description(String);

impl Description {
    /// Creates a new [`Description`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `description` is not empty.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(description: impl Into<String>) -> Self {
        Self(description.into())
    }

    /// Creates a new [`Description`] if the given `name` is valid.
    #[must_use]
    pub fn new(description: impl Into<String>) -> Option<Self> {
        let description = description.into();
        Self::check(&description).then_some(Self(description))
    }

    /// Checks whether the given `description` is a valid [`Description`].
    fn check(description: impl AsRef<str>) -> bool {
        let description = description.as_ref();
        description.trim() == description
            && !description.is_empty()
            && description.len() <= 512
    }
}

impl FromStr for Description {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or("invalid `Description`")
    }
}

define_kind! {
    #[doc = "Kind of a [`Contract`]."]
    enum Kind {
        #[doc = "[`Rent`] [`Contract`]."]
        Rent = 1,

        #[doc = "[`Sale`] [`Contract`]."]
        Sale = 2,

        #[doc = "[`ManagementForRent`] [`Contract`]."]
        ManagementForRent = 3,

        #[doc = "[`ManagementForSale`] [`Contract`]."]
        ManagementForSale = 4,

        #[doc = "[`Employment`] [`Contract`]."]
        Employment = 5,
    }
}

/// Status of a [`Contract`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Status {
    /// The [`Contract`] is active.
    Active = 1,

    /// The [`Contract`] is completed.
    Completed = 2,

    /// The [`Contract`] is terminated.
    Terminated = 3,
}

/// [`DateTime`] when a [`Contract`] was created.
pub type CreationDateTime = DateTimeOf<(Contract, unit::Creation)>;

/// Marker type indicating [`Contract`] expiration.
#[derive(Clone, Copy, Debug)]
pub struct Expiration;

/// [`DateTime`] when a [`Contract`] was expired.
pub type ExpirationDateTime = DateTimeOf<(Contract, Expiration)>;

/// [`DateTime`] when a [`Contract`] was terminated.
pub type TerminationDateTime = DateTimeOf<(Contract, unit::Deletion)>;
