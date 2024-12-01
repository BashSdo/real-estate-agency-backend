//! [`Contract`]-related definitions.

mod employment;
mod management_for_rent;
mod management_for_sale;
mod rent;
mod sale;

use common::DateTime;
use derive_more::{AsRef, Display, From, Into};
use juniper::{GraphQLInterface, GraphQLScalar};
use service::{domain, read};
use uuid::Uuid;

use crate::{api::scalar, Context};

pub use self::{
    employment::Employment, management_for_rent::ManagementForRent,
    management_for_sale::ManagementForSale, rent::Rent, sale::Sale,
};

/// Contract representing a legal agreement between two or more parties.
#[derive(Clone, Debug, GraphQLInterface)]
#[graphql(
    context = Context,
    for = [
        Employment,
        ManagementForRent,
        ManagementForSale,
        Rent,
        Sale,
    ]
)]
pub struct Contract {
    /// Unique identifier of the `Contract`.
    id: Id,

    /// Name of the `Contract`.
    name: Name,

    /// Description of the `Contract`.
    description: Description,

    /// `DateTime` when this `Contract` was created.
    created_at: DateTime,

    /// `DateTime` when this `Contract` expires.
    expires_at: Option<DateTime>,

    /// `DateTime` when this `Contract` was terminated.
    terminated_at: Option<DateTime>,
}

impl From<domain::Contract> for ContractValue {
    fn from(contract: domain::Contract) -> Self {
        use domain::Contract;
        match contract {
            Contract::Employment(c) => Self::Employment(c.into()),
            Contract::ManagementForRent(c) => Self::ManagementForRent(c.into()),
            Contract::ManagementForSale(c) => Self::ManagementForSale(c.into()),
            Contract::Rent(c) => Self::Rent(c.into()),
            Contract::Sale(c) => Self::Sale(c.into()),
        }
    }
}

impl From<read::contract::Active<domain::Contract>> for ContractValue {
    fn from(
        read::contract::Active(c): read::contract::Active<domain::Contract>,
    ) -> Self {
        c.into()
    }
}

impl ContractValue {
    /// Creates a new [`ContractValue`] from the provided [`Id`] and [`Kind`].
    ///
    /// # Safety
    ///
    /// Caller must ensure that [`Contract`] with the provided ID exists,
    /// otherwise accessing this [`Contract`] will result with an error.
    ///
    /// [`Kind`]: domain::contract::Kind
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(
        id: impl Into<Id>,
        kind: domain::contract::Kind,
    ) -> Self {
        use domain::contract::Kind;
        match kind {
            Kind::Employment => Self::Employment(Employment::new_unchecked(id)),
            Kind::ManagementForRent => {
                Self::ManagementForRent(ManagementForRent::new_unchecked(id))
            }
            Kind::ManagementForSale => {
                Self::ManagementForSale(ManagementForSale::new_unchecked(id))
            }
            Kind::Rent => Self::Rent(Rent::new_unchecked(id)),
            Kind::Sale => Self::Sale(Sale::new_unchecked(id)),
        }
    }
}

/// Unique identifier of a `Contract`.
#[derive(Clone, Copy, Debug, Display, Into, From, GraphQLScalar)]
#[from(domain::contract::Id)]
#[into(domain::contract::Id)]
#[graphql(name = "ContractId", transparent)]
pub struct Id(Uuid);

/// Name of a `Contract`.
#[derive(AsRef, Clone, Debug, Display, From, GraphQLScalar, Into)]
#[graphql(
    name = "ContractName",
    with = scalar::Via::<domain::contract::Name>,
)]
pub struct Name(domain::contract::Name);

/// Description of a `Contract`.
#[derive(AsRef, Clone, Debug, Display, From, GraphQLScalar, Into)]
#[graphql(
    name = "ContractDescription",
    with = scalar::Via::<domain::contract::Description>,
)]
pub struct Description(domain::contract::Description);

pub mod list {
    //! Definitions related to the [`Contract`] list.

    use derive_more::{AsRef, From, Into};
    use juniper::{graphql_object, GraphQLScalar};
    use service::{query, read, Query as _};

    #[cfg(doc)]
    use crate::api::Contract;
    use crate::{api::scalar, AsError, Context, Error};

    use super::{ContractValue, Id};

    /// Cursor for the `Contract` list.
    #[derive(AsRef, Clone, Copy, Debug, From, GraphQLScalar, Into)]
    #[from(Id, read::contract::list::Cursor)]
    #[graphql(
        name = "ContractListCursor",
        with = scalar::Via::<read::contract::list::Cursor>,
    )]
    pub struct Cursor(pub read::contract::list::Cursor);

    /// Edge in the [`Contract`] list.
    #[derive(Clone, Copy, Debug, From, Into)]
    pub struct Edge(read::contract::list::Edge);

    /// Edge in the `Contract` list.
    #[graphql_object(name = "ContractListEdge", context = Context)]
    impl Edge {
        /// Cursor of this `ContractListEdge`.
        #[must_use]
        pub fn cursor(&self) -> Cursor {
            self.0.cursor.into()
        }

        /// Node of this `ContractListEdge`.
        #[must_use]
        pub fn node(&self) -> ContractValue {
            let (id, kind) = self.0.node;

            #[expect(
                unsafe_code,
                reason = "`Edge` loaded from repository guarantees `Contract`\
                          existence"
            )]
            unsafe {
                ContractValue::new_unchecked(id, kind)
            }
        }
    }

    /// Connection of the [`Contract`] list.
    #[derive(Clone, Debug, From, Into)]
    pub struct Connection(read::contract::list::Connection);

    /// Connection of the `Contract` list.
    #[graphql_object(name = "ContractListConnection", context = Context)]
    impl Connection {
        /// Edges in this `ContractListConnection`.
        #[must_use]
        pub fn edges(&self) -> Vec<Edge> {
            self.0.edges.iter().copied().map(Into::into).collect()
        }

        /// Information about the page.
        #[must_use]
        pub fn page_info(&self) -> PageInfo {
            PageInfo {
                info: self.0.page_info(),
                start_cursor: self.0.edges.first().map(|e| e.cursor.into()),
                end_cursor: self.0.edges.last().map(|e| e.cursor.into()),
            }
        }
    }

    /// Information about a [`Connection`] page.
    #[derive(Clone, Copy, Debug)]
    pub struct PageInfo {
        /// Underlying [`read::contract::list::PageInfo`].
        info: read::contract::list::PageInfo,

        /// Start cursor of the page.
        start_cursor: Option<Cursor>,

        /// End cursor of the page.
        end_cursor: Option<Cursor>,
    }

    /// Information about a `ContractListConnection` page.
    #[graphql_object(name = "ContractListPageInfo", context = Context)]
    impl PageInfo {
        /// Indicator whether there is a next page.
        #[must_use]
        pub fn has_next_page(&self) -> bool {
            self.info.has_next_page
        }

        /// Indicator whether there is a previous page.
        #[must_use]
        pub fn has_previous_page(&self) -> bool {
            self.info.has_previous_page
        }

        /// Start cursor of the page.
        #[must_use]
        pub fn start_cursor(&self) -> &Option<Cursor> {
            &self.start_cursor
        }

        /// End cursor of the page.
        #[must_use]
        pub fn end_cursor(&self) -> &Option<Cursor> {
            &self.end_cursor
        }

        /// Total `Contract`s count.
        pub async fn total_count(&self, ctx: &Context) -> Result<i32, Error> {
            ctx.service()
                .execute(query::contracts::TotalCount::by(()))
                .await
                .map_err(AsError::into_error)
                .map_err(ctx.error())
                .map(Into::into)
        }
    }
}
