//! GraphQL scalar definitions.

use std::{fmt, marker::PhantomData, str::FromStr};

use juniper::{
    GraphQLType, InputValue, ParseScalarResult, ParseScalarValue, ScalarToken,
    ScalarValue, Value,
};

/// Helper type to use in `#[graphql(with = ..)]` attribute.
///
/// Uses [`FromStr`]/[`Display`] impls of `As` type to convert the target type
/// to/from GraphQL scalar.
///
/// Target type must implement [`TryFrom`] and [`AsRef`] for `As` type.
///
/// [`Display`]: fmt::Display
#[derive(Debug)]
pub struct Via<As>(PhantomData<As>);

impl<As> Via<As> {
    /// Convert the target type into scalar [`Value`] by using [`Display`] impl
    /// of `As` type.
    ///
    /// [`Display`]: fmt::Display
    pub fn to_output<T, S>(value: &T) -> Value<S>
    where
        As: fmt::Display,
        T: AsRef<As>,
        S: ScalarValue,
    {
        Value::from(value.as_ref().to_string())
    }

    /// Constructs the target type from scalar [`Value`] by using [`FromStr`]
    /// impl of `As` type.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - the input value is not a string;
    /// - the input value cannot be parsed into `As` type;
    /// - the parsed value cannot be converted into the target type.
    #[expect(clippy::missing_panics_doc, reason = "infallible")]
    pub fn from_input<T, S>(input: &InputValue<S>) -> Result<T, String>
    where
        As: FromStr + fmt::Display,
        As::Err: fmt::Display,
        T: TryFrom<As> + GraphQLType<S, TypeInfo = ()>,
        T::Error: fmt::Display,
        S: ScalarValue,
    {
        let s = input.as_string_value().ok_or_else(|| {
            format!(
                "Cannot parse input scalar `{}`: expected string input \
                 value, found: {input}",
                T::name(&()).expect("always has a name"),
            )
        })?;
        s.parse::<As>()
            .map_err(|e| {
                format!(
                    "Cannot parse input scalar `{}` from \"{s}\" string: {e}",
                    T::name(&()).expect("always has a name"),
                )
            })?
            .try_into()
            .map_err(|e| {
                format!(
                    "Cannot parse input scalar `{}`: {e}",
                    T::name(&()).expect("always has a name"),
                )
            })
    }

    /// Parse the provided [`ScalarToken`].
    ///
    /// # Errors
    ///
    /// Returns an error if the token cannot be parsed as [`String`].
    pub fn parse_token<S: ScalarValue>(
        value: ScalarToken<'_>,
    ) -> ParseScalarResult<S> {
        <String as ParseScalarValue<S>>::from_str(value)
    }
}
