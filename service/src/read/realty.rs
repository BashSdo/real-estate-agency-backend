//! [`Realty`]-related read definitions.

use derive_more::Deref;

#[cfg(doc)]
use crate::domain::Realty;

/// Indicator whether a [`Realty`] is rented or not.
#[derive(Clone, Copy, Debug, Deref, Eq, Hash, PartialEq)]
pub struct IsRented(pub bool);

impl PartialEq<bool> for IsRented {
    fn eq(&self, other: &bool) -> bool {
        self.0 == *other
    }
}

pub mod list {
    //! [`Realty`] list definitions.

    use common::define_pagination;
    use derive_more::{From, Into};

    use crate::domain::realty;
    #[cfg(doc)]
    use crate::domain::Realty;

    define_pagination!(Cursor, Node, Filter);

    /// Node in a [`Connection`].
    pub type Node = realty::Id;

    /// Cursor pointing to a specific [`Realty`] in a list.
    pub type Cursor = realty::Id;

    /// Filter for [`Selector`].
    #[derive(Clone, Debug, Default)]
    pub struct Filter {
        /// [`realty::Address`] (or its part) to fuzzy search for.
        pub address: Option<realty::Address>,
    }

    /// Total count of [`Realty`] list items.
    #[derive(Clone, Copy, Debug, Eq, From, Hash, Into, PartialEq)]
    pub struct TotalCount(i32);
}
