//! [`Contract`] read model definition.

#[cfg(doc)]
use crate::domain::Contract;

/// Wrapper around [`Contract`] indicating that it [`is_active()`].
///
/// [`is_active()`]: Contract::is_active
#[derive(Clone, Copy, Debug)]
pub struct Active<T>(pub T);

pub mod list {
    //! [`Contract`]s list definitions.

    use std::ops;

    use common::define_pagination;
    use derive_more::{From, Into};

    use crate::domain::contract;
    #[cfg(doc)]
    use crate::domain::Contract;

    define_pagination!(Cursor, Node, Filter);

    /// Node in a [`Connection`].
    pub type Node = (contract::Id, contract::Kind);

    /// Cursor pointing to a specific [`Contract`] in a list.
    pub type Cursor = contract::Id;

    /// Filter for [`Selector`].
    #[derive(Clone, Debug, Default)]
    pub struct Filter {
        /// [`contract::Name`] (or its part) to fuzzy search for.
        pub name: Option<contract::Name>,
    }

    /// Total count of [`Contract`]s.
    #[derive(Clone, Copy, Debug, Eq, From, Hash, Into, PartialEq)]
    pub struct TotalCount(i32);

    impl ops::Div for TotalCount {
        type Output = f64;

        fn div(self, rhs: Self) -> Self::Output {
            f64::from(self.0) / f64::from(rhs.0)
        }
    }
}
