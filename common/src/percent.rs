//! [`Percent`]-related definitions.

use std::str::FromStr;

use derive_more::Display;
#[cfg(feature = "postgres")]
use postgres_types::{FromSql, ToSql};
use rust_decimal::Decimal;

/// Floating-point percentage.
#[derive(Clone, Copy, Debug, Display, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "postgres", derive(FromSql, ToSql), postgres(transparent))]
pub struct Percent(Decimal);

impl Percent {
    /// Creates a new [`Percent`] by checking the provided values is
    /// greater than `0` and less than `100`.
    #[must_use]
    pub fn new(val: Decimal) -> Option<Self> {
        if val < Decimal::ZERO || val > Decimal::ONE_HUNDRED {
            None
        } else {
            #[expect(
                clippy::allow_attributes,
                reason = "TODO: Remove once clippy is fixed"
            )]
            #[allow(unsafe_code, reason = "invariants checked already")]
            Some(unsafe { Self::new_unchecked(val) })
        }
    }

    /// Creates a new [`Percent`] without performing any validation.
    ///
    /// # Safety
    ///
    /// The provided value must be greater than `0` and less than `100`.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(val: Decimal) -> Self {
        Self(val)
    }
}

impl FromStr for Percent {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Decimal::from_str(s)
            .ok()
            .and_then(Self::new)
            .ok_or("invalid percent value")
    }
}

#[cfg(feature = "juniper")]
mod juniper {
    //! Module providing integration with [`juniper`] crate.

    use std::str::FromStr as _;

    use juniper::{graphql_scalar, InputValue, ScalarValue, Value};

    /// Floating-point percentage.
    #[graphql_scalar(with = Self, parse_token(String))]
    type Percent = super::Percent;

    impl Percent {
        fn to_output<S: ScalarValue>(m: &Percent) -> Value<S> {
            Value::scalar(m.to_string())
        }

        fn from_input<S: ScalarValue>(
            input: &InputValue<S>,
        ) -> Result<Self, String> {
            input
                .as_string_value()
                .ok_or_else(|| {
                    format!(
                        "Cannot parse `Percent` input scalar from \
                         non-string value: {input}",
                    )
                })
                .and_then(|s| {
                    Self::from_str(s).map_err(|e| {
                        format!("Cannot parse `Percent` input scalar: {e}")
                    })
                })
        }
    }
}
