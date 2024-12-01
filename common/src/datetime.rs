//! Date and time utilities.

#[cfg(feature = "postgres")]
use std::error::Error as StdError;
use std::{cmp::Ordering, marker::PhantomData, ops, time::Duration};

use derive_more::{Debug, Display, Error};
#[cfg(feature = "postgres")]
use postgres_types::{
    accepts, private::BytesMut, to_sql_checked, FromSql, IsNull, ToSql, Type,
};
use time::{format_description::well_known::Rfc3339, UtcOffset};

/// Untyped date and time.
pub type DateTime = DateTimeOf;

/// UTC date and time.
#[derive(Debug)]
pub struct DateTimeOf<Of: ?Sized = ()> {
    /// Inner representation of the date and time.
    inner: time::OffsetDateTime,

    /// Type parameter describing the kind of date and time.
    #[debug(skip)]
    _of: PhantomData<Of>,
}

impl<Of: ?Sized> DateTimeOf<Of> {
    /// A [`DateTime`] representing the Unix epoch.
    pub const UNIX_EPOCH: Self = Self {
        inner: time::OffsetDateTime::UNIX_EPOCH,
        _of: PhantomData,
    };

    /// Creates a new [`DateTime`] representing the current date and time.
    #[expect(clippy::missing_panics_doc, reason = "infallible")]
    #[must_use]
    pub fn now() -> Self {
        let inner = time::OffsetDateTime::now_utc();
        Self {
            _of: PhantomData,
            inner: inner
                .replace_microsecond(inner.microsecond())
                .expect("infallible"),
        }
    }

    /// Creates a new [`DateTime`] from the provided [`UNIX_EPOCH`] timestamp.
    ///
    /// [`None`] is returned if the timestamp is invalid.
    ///
    /// [`UNIX_EPOCH`]: Self::UNIX_EPOCH
    #[must_use]
    pub fn from_unix_timestamp(timestamp: i64) -> Option<Self> {
        Some(Self {
            inner: time::OffsetDateTime::from_unix_timestamp(timestamp).ok()?,
            _of: PhantomData,
        })
    }

    /// Returns the [`UNIX_EPOCH`] timestamp of this [`DateTime`].
    ///
    /// [`UNIX_EPOCH`]: Self::UNIX_EPOCH
    #[must_use]
    pub fn unix_timestamp(&self) -> i64 {
        self.inner.unix_timestamp()
    }

    /// Creates a new [`DateTime`] from the provided [RFC 3339] string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not a valid [RFC 3339] date and time.
    ///
    /// [RFC 3339]: https://tools.ietf.org/html/rfc3339
    pub fn from_rfc3339(input: &str) -> Result<Self, ParseError> {
        use ParseError as E;

        time::OffsetDateTime::parse(input, &Rfc3339)
            .map_err(E::Parse)?
            .try_into()
            .map_err(E::ComponentRange)
    }

    /// Returns the [`DateTime`] as an [RFC 3339] string.
    ///
    /// [RFC 3339]: https://tools.ietf.org/html/rfc3339
    #[expect(clippy::missing_panics_doc, reason = "infallible")]
    #[must_use]
    pub fn to_rfc3339(&self) -> String {
        self.inner.format(&Rfc3339).unwrap_or_else(|e| {
            panic!("cannot format `DateTime` as RFC 3339: {e}")
        })
    }

    /// Coerces one kind of [`DateTime`] into another.
    #[must_use]
    pub fn coerce<NewOf: ?Sized>(self) -> DateTimeOf<NewOf> {
        DateTimeOf {
            inner: self.inner,
            _of: PhantomData,
        }
    }
}

/// Error of parsing [`DateTime`] from a string.
#[derive(Clone, Copy, Debug, Display, Error)]
pub enum ParseError {
    /// Failed to parse the string into an [`DateTime`].
    Parse(time::error::Parse),

    /// Parsed [`DateTime`] has an out of range component.
    ComponentRange(time::error::ComponentRange),
}

impl<Of: ?Sized> Copy for DateTimeOf<Of> {}
impl<Of: ?Sized> Clone for DateTimeOf<Of> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Of: ?Sized> Eq for DateTimeOf<Of> {}
impl<Of: ?Sized> PartialEq for DateTimeOf<Of> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<Of: ?Sized> Ord for DateTimeOf<Of> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}
impl<Of: ?Sized> PartialOrd for DateTimeOf<Of> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Of: ?Sized> TryFrom<time::OffsetDateTime> for DateTimeOf<Of> {
    type Error = time::error::ComponentRange;

    fn try_from(dt: time::OffsetDateTime) -> Result<Self, Self::Error> {
        dt.to_offset(UtcOffset::UTC)
            .replace_microsecond(dt.microsecond())
            .map(|inner| Self {
                inner,
                _of: PhantomData,
            })
    }
}

impl<Of: ?Sized> From<DateTimeOf<Of>> for time::OffsetDateTime {
    fn from(dt: DateTimeOf<Of>) -> Self {
        dt.inner
    }
}

impl<Of: ?Sized> ops::Add<Duration> for DateTimeOf<Of> {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Self {
            inner: self.inner + rhs,
            _of: PhantomData,
        }
    }
}

impl<Of: ?Sized> ops::Sub for DateTimeOf<Of> {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        (self.inner - rhs.inner)
            .try_into()
            .expect("duration overflow")
    }
}

impl<Of: ?Sized> ops::Sub<Duration> for DateTimeOf<Of> {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self {
            inner: self.inner - rhs,
            _of: PhantomData,
        }
    }
}

#[cfg(feature = "postgres")]
impl<Of: ?Sized> FromSql<'_> for DateTimeOf<Of> {
    accepts!(TIMESTAMPTZ);

    fn from_sql(
        ty: &Type,
        raw: &[u8],
    ) -> Result<Self, Box<dyn StdError + Sync + Send>> {
        time::OffsetDateTime::from_sql(ty, raw)?
            .try_into()
            .map_err(Box::from)
    }
}

#[cfg(feature = "postgres")]
impl<Of: ?Sized> ToSql for DateTimeOf<Of> {
    accepts!(TIMESTAMPTZ);
    to_sql_checked!();

    fn to_sql(
        &self,
        ty: &Type,
        w: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn StdError + Sync + Send>> {
        self.inner.to_sql(ty, w)
    }
}

#[cfg(feature = "serde")]
pub mod serde {
    //! Module providing integration with [`serde`] crate.

    use super::DateTimeOf;

    pub mod unix_timestamp {
        //! Module providing serialization and deserialization of [`DateTimeOf`]
        //! as a Unix timestamp.

        use serde::{de::Error, Deserialize, Deserializer, Serializer};

        use super::DateTimeOf;

        /// Serializes the [`DateTimeOf`] as a Unix timestamp.
        ///
        /// # Errors
        ///
        /// Returns an error if the timestamp is invalid.
        pub fn serialize<Of, S>(
            dt: &DateTimeOf<Of>,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
            Of: ?Sized,
        {
            serializer.serialize_i64(dt.unix_timestamp())
        }

        /// Deserializes the Unix timestamp into a [`DateTimeOf`].
        ///
        /// # Errors
        ///
        /// Returns an error if the timestamp is invalid.
        pub fn deserialize<'de, D, Of>(
            deserializer: D,
        ) -> Result<DateTimeOf<Of>, D::Error>
        where
            D: Deserializer<'de>,
            Of: ?Sized,
        {
            DateTimeOf::from_unix_timestamp(i64::deserialize(deserializer)?)
                .ok_or_else(|| Error::custom("invalid timestamp"))
        }
    }
}

#[cfg(feature = "juniper")]
mod juniper {
    //! Module providing integration with [`juniper`] crate.

    use juniper::{graphql_scalar, InputValue, ScalarValue, Value};

    /// Date and time in a [RFC 3339] format with a microsecond precision.
    ///
    /// [RFC 3339]: https://tools.ietf.org/html/rfc3339
    #[graphql_scalar(with = Self, parse_token(String))]
    type DateTime = crate::DateTime;

    impl DateTime {
        fn to_output<S: ScalarValue>(dt: &DateTime) -> Value<S> {
            Value::scalar(dt.to_rfc3339())
        }

        fn from_input<S: ScalarValue>(
            input: &InputValue<S>,
        ) -> Result<Self, String> {
            input
                .as_string_value()
                .ok_or_else(|| {
                    format!(
                        "Cannot parse `DateTime` input scalar from \
                         non-string value: {input}",
                    )
                })
                .and_then(|s| {
                    Self::from_rfc3339(s).map_err(|e| {
                        format!("Cannot parse `DateTime` input scalar: {e}")
                    })
                })
        }
    }
}
