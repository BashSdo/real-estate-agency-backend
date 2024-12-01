//! [`FuzzPattern`] definition.

use derive_more::Display;
use itertools::Itertools as _;
use postgres_types::{FromSql, ToSql};

/// SQL pattern to be used for fuzzy searching.
#[derive(Clone, Debug, Display, Eq, FromSql, PartialEq, ToSql)]
#[postgres(transparent)]
pub struct FuzzPattern(String);

impl FuzzPattern {
    /// Creates a new [`FuzzPattern`] out of the given `input`.
    #[must_use]
    pub fn new(input: &str) -> Self {
        Self(format!(
            "({})",
            input.split_ascii_whitespace().format_with("|", |word, f| {
                f(&format_args!(
                    "%{}%",
                    word.replace('\\', r"\\")
                        .replace('%', r"\%")
                        .replace('|', r"\|")
                        .replace('*', r"\*")
                        .replace('+', r"\+")
                        .replace('?', r"\?")
                        .replace('{', r"\{")
                        .replace('}', r"\}")
                        .replace('(', r"\(")
                        .replace(')', r"\)")
                        .replace('[', r"\[")
                        .replace(']', r"\]")
                        .replace('_', r"\_")
                ))
            }),
        ))
    }
}
