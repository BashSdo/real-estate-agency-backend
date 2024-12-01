//! [`Placement`] read model definition.

#[cfg(doc)]
use crate::domain::Realty;
use crate::domain::{contract, realty};

/// Placement of a [`Realty`] in the real estate market.
#[derive(Clone, Copy, Debug)]
pub struct Placement {
    /// ID of the placed [`Realty`].
    pub realty_id: realty::Id,

    /// ID of the [`contract::ManagementForRent`] related to this [`Placement`].
    pub rent_contract_id: Option<contract::Id>,

    /// ID of the [`contract::ManagementForSale`] related to this [`Placement`].
    pub sale_contract_id: Option<contract::Id>,
}

pub mod list {
    //! [`Placement`]s list definitions.

    use common::define_pagination;
    use derive_more::{From, Into};
    use smart_default::SmartDefault;

    use crate::domain::realty;

    use super::Placement;

    define_pagination!(Cursor, Node, Filter);

    /// Node in a [`Connection`].
    pub type Node = Placement;

    /// Cursor pointing to a specific [`Placement`] in a list.
    pub type Cursor = realty::Id;

    /// Filter for [`Selector`].
    #[derive(Clone, Copy, Debug, SmartDefault)]
    pub struct Filter {
        /// Include sale [`Placement`].
        #[default(true)]
        pub sale: bool,

        /// Include rent [`Placement`].
        #[default(true)]
        pub rent: bool,
    }

    /// Total count of [`Placement`]s.
    #[derive(Clone, Copy, Debug, Eq, From, Hash, Into, PartialEq)]
    pub struct TotalCount(i32);
}
