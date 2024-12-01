//! [`User`] read model definition.
//!
//! [`User`]: crate::domain::User

pub mod list {
    //! [`User`]s list definitions.

    use common::define_pagination;
    use derive_more::{From, Into};

    use crate::domain::user;
    #[cfg(doc)]
    use crate::domain::User;

    define_pagination!(Cursor, Node, Filter);

    /// Node in a [`Connection`].
    pub type Node = user::Id;

    /// Cursor pointing to a specific [`User`] in a list.
    pub type Cursor = user::Id;

    /// Filter for [`Selector`].
    #[derive(Clone, Debug, Default)]
    pub struct Filter {
        /// [`user::Name`] (or its part) to fuzzy search for.
        pub name: Option<user::Name>,
    }

    /// Total count of [`User`]s.
    #[derive(Clone, Copy, Debug, Eq, From, Hash, Into, PartialEq)]
    pub struct TotalCount(i32);
}
