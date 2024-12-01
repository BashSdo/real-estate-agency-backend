//! [`Realty`] definitions.

#[cfg(doc)]
use common::DateTime;
use common::{define_kind, unit, DateTimeOf};
use derive_more::{AsRef, Display, From, FromStr, Into};
#[cfg(feature = "postgres")]
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use xxhash_rust::xxh3;

/// Realty for rent or sale.
#[derive(Clone, Debug)]
pub struct Realty {
    /// ID of this [`Realty`].
    pub id: Id,

    /// [`Hash`] of this [`Realty`] used for deduplication.
    ///
    /// [`Hash`]: struct@Hash
    pub hash: Hash,

    /// [`Address`] of this [`Realty`].
    pub address: Address,

    /// [`Country`] this [`Realty`] is located.
    pub country: Country,

    /// [`State`] this [`Realty`] is located.
    pub state: Option<State>,

    /// [`City`] this [`Realty`] is located.
    pub city: City,

    /// [`Street`] this [`Realty`] is located.
    pub street: Street,

    /// [`ZipCode`] of this [`Realty`].
    pub zip_code: Option<ZipCode>,

    /// [`BuildingName`] of this [`Realty`].
    pub building_name: BuildingName,

    /// Number of floors in this [`Realty`].
    pub num_floors: NumFloors,

    /// Floor of this [`Realty`], if any.
    pub floor: Option<Floor>,

    /// Apartment number of this [`Realty`], if any.
    pub apartment_num: Option<ApartmentNum>,

    /// Room number of this [`Realty`], if any.
    pub room_num: Option<RoomNum>,

    /// [`DateTime`] when this [`Realty`] was created.
    pub created_at: CreationDateTime,

    /// [`DateTime`] when this [`Realty`] was deleted, if it was.
    pub deleted_at: Option<DeletionDateTime>,
}

impl Realty {
    /// Returns [`Kind`] of this [`Realty`].
    #[must_use]
    pub fn kind(&self) -> Kind {
        if self.room_num.is_some() {
            return Kind::Room;
        }

        if self.apartment_num.is_some() {
            return Kind::Apartment;
        }

        Kind::Building
    }
}

/// ID of a [`Realty`].
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

/// Hash of a [`Realty`] used for deduplication.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    Eq,
    From,
    Hash,
    Into,
    PartialEq,
    Serialize,
)]
#[cfg_attr(feature = "postgres", derive(ToSql, FromSql), postgres(transparent))]
pub struct Hash(Uuid);

impl Hash {
    /// Calculates a new [`Hash`] for a [`Realty`].
    ///
    /// [`Hash`]: struct@Hash
    #[expect(clippy::too_many_arguments, reason = "still readable")]
    #[must_use]
    pub fn new(
        country: &Country,
        state: Option<&State>,
        city: &City,
        street: &Street,
        zip_code: Option<&ZipCode>,
        building_name: &BuildingName,
        num_floors: NumFloors,
        floor: Option<Floor>,
        apartment_num: Option<&ApartmentNum>,
        room_num: Option<&RoomNum>,
    ) -> Self {
        use std::hash::Hash as _;

        // WARNING: Avoid changing the order of the fields in the hasher,
        //          because it will be a breaking change requiring to migrate
        //          all existing hashes in the database to the new format.
        let mut hasher = xxh3::Xxh3Builder::new().build();
        country.hash(&mut hasher);
        state.hash(&mut hasher);
        city.hash(&mut hasher);
        street.hash(&mut hasher);
        zip_code.hash(&mut hasher);
        building_name.hash(&mut hasher);
        num_floors.hash(&mut hasher);
        floor.hash(&mut hasher);
        apartment_num.hash(&mut hasher);
        room_num.hash(&mut hasher);

        Self(Uuid::from_u128(hasher.digest128()))
    }
}

/// Country of a [`Realty`].
#[derive(AsRef, Clone, Debug, Display, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "postgres", derive(FromSql, ToSql), postgres(transparent))]
#[as_ref(forward)]
pub struct Country(String);

impl Country {
    /// Creates a new [`Country`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `country` matches the format.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(country: impl Into<String>) -> Self {
        Self(country.into())
    }

    /// Creates a new [`Country`] if the given `country` is valid.
    #[must_use]
    pub fn new(country: impl Into<String>) -> Option<Self> {
        let country = country.into();
        Self::check(&country).then_some(Self(country))
    }

    /// Checks whether the given `country` is a valid [`Country`].
    fn check(country: impl AsRef<str>) -> bool {
        let country = country.as_ref();
        country.trim() == country && !country.is_empty() && country.len() <= 512
    }
}

impl FromStr for Country {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or("invalid `Country`")
    }
}

/// State of a [`Realty`].
#[derive(AsRef, Clone, Debug, Display, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "postgres", derive(FromSql, ToSql), postgres(transparent))]
#[as_ref(forward)]
pub struct State(String);

impl State {
    /// Creates a new [`State`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `state` matches the format.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(state: impl Into<String>) -> Self {
        Self(state.into())
    }

    /// Creates a new [`State`] if the given `state` is valid.
    #[must_use]
    pub fn new(state: impl Into<String>) -> Option<Self> {
        let state = state.into();
        Self::check(&state).then_some(Self(state))
    }

    /// Checks whether the given `state` is a valid [`State`].
    fn check(state: impl AsRef<str>) -> bool {
        let state = state.as_ref();
        state.trim() == state && !state.is_empty() && state.len() <= 512
    }
}

impl FromStr for State {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or("invalid `State`")
    }
}

/// City of a [`Realty`].
#[derive(AsRef, Clone, Debug, Display, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "postgres", derive(FromSql, ToSql), postgres(transparent))]
#[as_ref(forward)]
pub struct City(String);

impl City {
    /// Creates a new [`City`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `city` matches the format.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(city: impl Into<String>) -> Self {
        Self(city.into())
    }

    /// Creates a new [`City`] if the given `city` is valid.
    #[must_use]
    pub fn new(city: impl Into<String>) -> Option<Self> {
        let city = city.into();
        Self::check(&city).then_some(Self(city))
    }

    /// Checks whether the given `city` is a valid [`City`].
    fn check(city: impl AsRef<str>) -> bool {
        let city = city.as_ref();
        city.trim() == city && !city.is_empty() && city.len() <= 512
    }
}

impl FromStr for City {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or("invalid `City`")
    }
}

/// Street of a [`Realty`].
#[derive(AsRef, Clone, Debug, Display, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "postgres", derive(FromSql, ToSql), postgres(transparent))]
#[as_ref(forward)]
pub struct Street(String);

impl Street {
    /// Creates a new [`Street`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `street` matches the format.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(street: impl Into<String>) -> Self {
        Self(street.into())
    }

    /// Creates a new [`Street`] if the given `street` is valid.
    #[must_use]
    pub fn new(street: impl Into<String>) -> Option<Self> {
        let street = street.into();
        Self::check(&street).then_some(Self(street))
    }

    /// Checks whether the given `street` is a valid [`Street`].
    fn check(street: impl AsRef<str>) -> bool {
        let street = street.as_ref();
        street.trim() == street && !street.is_empty() && street.len() <= 512
    }
}

impl FromStr for Street {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or("invalid `Street`")
    }
}

/// Zip code of a [`Realty`].
#[derive(AsRef, Clone, Debug, Display, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "postgres", derive(FromSql, ToSql), postgres(transparent))]
#[as_ref(forward)]
pub struct ZipCode(String);

impl ZipCode {
    /// Creates a new [`ZipCode`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `code` matches the format.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    /// Creates a new [`ZipCode`] if the given `code` is valid.
    #[must_use]
    pub fn new(code: impl Into<String>) -> Option<Self> {
        let code = code.into();
        Self::check(&code).then_some(Self(code))
    }

    /// Checks whether the given `code` is a valid [`ZipCode`].
    fn check(code: impl AsRef<str>) -> bool {
        let code = code.as_ref();
        code.trim() == code && !code.is_empty() && code.len() <= 512
    }
}

impl FromStr for ZipCode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or("invalid `ZipCode`")
    }
}

/// Building name of a [`Realty`].
#[derive(AsRef, Clone, Debug, Display, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "postgres", derive(FromSql, ToSql), postgres(transparent))]
#[as_ref(forward)]
pub struct BuildingName(String);

impl BuildingName {
    /// Creates a new [`BuildingName`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `name` matches the format.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Creates a new [`BuildingName`] if the given `name` is valid.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        Self::check(&name).then_some(Self(name))
    }

    /// Checks whether the given `name` is a valid [`BuildingName`].
    fn check(name: impl AsRef<str>) -> bool {
        let name = name.as_ref();
        name.trim() == name && !name.is_empty() && name.len() <= 512
    }
}

impl FromStr for BuildingName {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or("invalid `BuildingName`")
    }
}

/// Number of floors in a [`Realty`].
pub type NumFloors = u16;

/// Floor of a [`Realty`].
pub type Floor = u16;

/// Apartment number of a [`Realty`].
#[derive(AsRef, Clone, Debug, Display, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "postgres", derive(FromSql, ToSql), postgres(transparent))]
#[as_ref(forward)]
pub struct ApartmentNum(String);

impl ApartmentNum {
    /// Creates a new [`ApartmentNum`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `num` matches the format.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(num: impl Into<String>) -> Self {
        Self(num.into())
    }

    /// Creates a new [`ApartmentNum`] if the given `num` is valid.
    #[must_use]
    pub fn new(num: impl Into<String>) -> Option<Self> {
        let num = num.into();
        Self::check(&num).then_some(Self(num))
    }

    /// Checks whether the given `num` is a valid [`ApartmentNum`].
    fn check(num: impl AsRef<str>) -> bool {
        let num = num.as_ref();
        num.trim() == num && !num.is_empty() && num.len() <= 512
    }
}

impl FromStr for ApartmentNum {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or("invalid `ApartmentNum`")
    }
}

/// Room number of a [`Realty`].
#[derive(AsRef, Clone, Debug, Display, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "postgres", derive(FromSql, ToSql), postgres(transparent))]
#[as_ref(forward)]
pub struct RoomNum(String);

impl RoomNum {
    /// Creates a new [`RoomNum`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `num` matches the format.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(num: impl Into<String>) -> Self {
        Self(num.into())
    }

    /// Creates a new [`RoomNum`] if the given `num` is valid.
    #[must_use]
    pub fn new(num: impl Into<String>) -> Option<Self> {
        let num = num.into();
        Self::check(&num).then_some(Self(num))
    }

    /// Checks whether the given `num` is a valid [`RoomNum`].
    fn check(num: impl AsRef<str>) -> bool {
        let num = num.as_ref();
        num.trim() == num && !num.is_empty() && num.len() <= 512
    }
}

impl FromStr for RoomNum {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or("invalid `RoomNum`")
    }
}

define_kind! {
    #[doc = "Kind of a [`Realty`]."]
    enum Kind {
        #[doc = "An apartment in a building."]
        Apartment = 1,

        #[doc = "A whole building."]
        Building = 2,

        #[doc = "A room in a building."]
        Room = 3,
    }
}

/// Full address of a [`Realty`].
#[derive(AsRef, Clone, Debug, Display, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "postgres", derive(FromSql, ToSql), postgres(transparent))]
#[as_ref(forward)]
pub struct Address(String);

impl Address {
    /// Creates a new [`Address`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `address` matches the format.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(address: impl Into<String>) -> Self {
        Self(address.into())
    }

    /// Creates a new [`Address`] if the given `address` is valid.
    #[must_use]
    pub fn new(address: impl Into<String>) -> Option<Self> {
        let address = address.into();
        Self::check(&address).then_some(Self(address))
    }

    /// Creates a new [`Address`].
    #[expect(clippy::too_many_arguments, reason = "still readable")]
    #[must_use]
    pub fn from_parts(
        country: &Country,
        state: Option<&State>,
        city: &City,
        street: &Street,
        zip_code: Option<&ZipCode>,
        building_name: &BuildingName,
        floor: Option<Floor>,
        apartment_num: Option<&ApartmentNum>,
        room_num: Option<&RoomNum>,
    ) -> Self {
        let mut address = String::with_capacity(512);
        address.push_str(country.as_ref());
        if let Some(state) = state {
            address.push_str(", ");
            address.push_str(state.as_ref());
        }
        address.push_str(", ");
        address.push_str(city.as_ref());
        address.push_str(", ");
        address.push_str(street.as_ref());
        if let Some(zip_code) = zip_code {
            address.push_str(", ");
            address.push_str(zip_code.as_ref());
        }
        address.push_str(", ");
        address.push_str(building_name.as_ref());
        if let Some(floor) = floor {
            address.push_str(", ");
            address.push_str(&floor.to_string());
        }
        if let Some(apartment_num) = apartment_num {
            address.push_str(", ");
            address.push_str(apartment_num.as_ref());
        }
        if let Some(room_num) = room_num {
            address.push_str(", ");
            address.push_str(room_num.as_ref());
        }
        Self(address)
    }

    /// Checks whether the given `address` is a valid [`Address`].
    fn check(address: impl AsRef<str>) -> bool {
        let address = address.as_ref();
        address.trim() == address && !address.is_empty() && address.len() <= 512
    }
}

impl FromStr for Address {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or("invalid `Address`")
    }
}

/// [`DateTime`] when a [`Realty`] was created.
pub type CreationDateTime = DateTimeOf<(Realty, unit::Creation)>;

/// [`DateTime`] when a [`Realty`] was deleted.
pub type DeletionDateTime = DateTimeOf<(Realty, unit::Deletion)>;
