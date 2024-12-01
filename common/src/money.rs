//! [`Money`]-related definitions.

use std::{fmt, str::FromStr};

use rust_decimal::{prelude::ToPrimitive as _, Decimal};

use crate::define_kind;

/// Amount of money in some [`Currency`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Money {
    /// Amount of this [`Money`].
    pub amount: Decimal,

    /// [`Currency`] of this amount.
    pub currency: Currency,
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { amount, currency } = self;
        if amount.is_integer() {
            write!(f, "{}{currency}", amount.to_i128().expect("integer"))
        } else {
            write!(f, "{amount}{currency}")
        }
    }
}

impl FromStr for Money {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 4 {
            return Err("too short");
        }

        let (amount, currency) = s.split_at(s.len() - 3);
        let amount = Decimal::from_str(amount).map_err(|_| "invalid amount")?;
        let currency =
            Currency::from_str(currency).map_err(|_| "invalid currency")?;

        Ok(Self { amount, currency })
    }
}

define_kind! {
    #[doc = "Currency of a [`Money`] amount."]
    enum Currency {
        #[doc = "US Dollar."]
        Usd = 1,

        #[doc = "Euro."]
        Eur = 2,

        #[doc = "Russian Ruble."]
        Rub = 3,
    }
}

#[cfg(feature = "juniper")]
mod juniper {
    //! Module providing integration with [`juniper`] crate.

    use std::str::FromStr as _;

    use juniper::{graphql_scalar, InputValue, ScalarValue, Value};

    /// Money in `{major}.{minor}{currency}` format, where:
    /// - `major` is an integer;
    /// - `minor` is an optional integer;
    /// - `currency` is a three-letter currency code.
    #[graphql_scalar(with = Self, parse_token(String))]
    type Money = super::Money;

    impl Money {
        fn to_output<S: ScalarValue>(m: &Money) -> Value<S> {
            Value::scalar(m.to_string())
        }

        fn from_input<S: ScalarValue>(
            input: &InputValue<S>,
        ) -> Result<Self, String> {
            input
                .as_string_value()
                .ok_or_else(|| {
                    format!(
                        "Cannot parse `Money` input scalar from \
                         non-string value: {input}",
                    )
                })
                .and_then(|s| {
                    Self::from_str(s).map_err(|e| {
                        format!("Cannot parse `Money` input scalar: {e}")
                    })
                })
        }
    }
}

#[cfg(test)]
mod spec {
    use std::str::FromStr as _;

    use rust_decimal::Decimal;

    use super::{Currency, Money};

    fn decimal(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    #[test]
    fn from_str() {
        assert_eq!(
            Money::from_str("123.45USD").unwrap(),
            Money {
                amount: decimal("123.45"),
                currency: Currency::Usd,
            },
        );

        assert_eq!(
            Money::from_str("123.45EUR").unwrap(),
            Money {
                amount: decimal("123.45"),
                currency: Currency::Eur,
            },
        );

        assert_eq!(
            Money::from_str("123.45RUB").unwrap(),
            Money {
                amount: decimal("123.45"),
                currency: Currency::Rub,
            },
        );

        assert!(Money::from_str("123.45").is_err());
        assert!(Money::from_str("123.45Us").is_err());
        assert!(Money::from_str("123.45Usdollar").is_err());

        assert!(Money::from_str("123.00USD").is_ok());
        assert!(Money::from_str("123.0USD").is_ok());
        assert!(Money::from_str("123USD").is_ok());
    }

    #[test]
    fn to_string() {
        assert_eq!(
            Money {
                amount: decimal("123.45"),
                currency: Currency::Usd,
            }
            .to_string(),
            "123.45USD",
        );

        assert_eq!(
            Money {
                amount: decimal("123.45"),
                currency: Currency::Eur,
            }
            .to_string(),
            "123.45EUR",
        );

        assert_eq!(
            Money {
                amount: decimal("123.45"),
                currency: Currency::Rub,
            }
            .to_string(),
            "123.45RUB",
        );

        assert_eq!(
            Money {
                amount: decimal("123.00"),
                currency: Currency::Usd,
            }
            .to_string(),
            "123USD",
        );
        assert_eq!(
            Money {
                amount: decimal("123.0"),
                currency: Currency::Usd,
            }
            .to_string(),
            "123USD",
        );
        assert_eq!(
            Money {
                amount: decimal("123"),
                currency: Currency::Usd,
            }
            .to_string(),
            "123USD",
        );
    }
}
