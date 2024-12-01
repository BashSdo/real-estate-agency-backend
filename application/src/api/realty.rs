//! [`Realty`]-related definitions.

use std::future;

use common::{DateTime, Handler as _};
use derive_more::{AsRef, Display, From, Into};
use futures::TryFutureExt as _;
use juniper::{graphql_object, GraphQLEnum, GraphQLScalar};
use service::{domain, query};
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::{api, api::scalar, AsError, Context, Error};

/// A realty.
#[derive(Clone, Debug, From)]
pub struct Realty {
    /// ID of this [`Realty`].
    id: Id,

    /// Underlying [`domain::Realty`].
    realty: OnceCell<domain::Realty>,
}

impl From<domain::Realty> for Realty {
    fn from(realty: domain::Realty) -> Self {
        Self {
            id: realty.id.into(),
            realty: OnceCell::new_with(Some(realty)),
        }
    }
}

impl Realty {
    /// Creates a new [`Realty`] with the provided ID.
    ///
    /// # Safety
    ///
    /// Caller must ensure that [`Realty`] with the provided ID exists,
    /// otherwise accessing this [`Realty`] will result with an error.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(id: impl Into<Id>) -> Self {
        Self {
            id: id.into(),
            realty: OnceCell::new(),
        }
    }

    /// Returns the underlying [`domain::Realty`].
    ///
    /// # Errors
    ///
    /// Errors if the [`domain::Realty`] doesn't exist.
    async fn realty(&self, ctx: &Context) -> Result<&domain::Realty, Error> {
        let id = self.id.into();
        self.realty
            .get_or_try_init(|| {
                ctx.service()
                    .execute(query::realty::ById::by(id))
                    .map_err(AsError::into_error)
                    .map_err(ctx.error())
                    .and_then(|u| {
                        future::ready(u.ok_or_else(|| {
                            api::query::RealtyError::NotExists.into()
                        }))
                    })
            })
            .await
    }
}

/// A realty.
#[graphql_object(context = Context)]
impl Realty {
    /// Unique identifier of this `Realty`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "Realty.id",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub fn id(&self) -> Id {
        self.id
    }

    /// Kind of this `Realty`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "Realty.kind",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn kind(&self, ctx: &Context) -> Result<Kind, Error> {
        Ok(self.realty(ctx).await?.kind().into())
    }

    /// Address of this `Realty`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "Realty.address",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn address(&self, ctx: &Context) -> Result<Address, Error> {
        Ok(self.realty(ctx).await?.address.clone().into())
    }

    /// `DateTime` when this `Realty` was created.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "Realty.createdAt",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn created_at(&self, ctx: &Context) -> Result<DateTime, Error> {
        Ok(self.realty(ctx).await?.created_at.coerce())
    }
}

/// Unique identifier of a `Realty`.
#[derive(Clone, Copy, Debug, Display, Into, From, GraphQLScalar)]
#[from(domain::realty::Id)]
#[into(domain::realty::Id)]
#[graphql(name = "RealtyId", transparent)]
pub struct Id(Uuid);

/// Address of a `Realty`.
#[derive(AsRef, Clone, Debug, Display, From, GraphQLScalar, Into)]
#[graphql(
    name = "RealtyAddress",
    with = scalar::Via::<domain::realty::Address>,
)]
pub struct Address(domain::realty::Address);

/// Country of a `Realty`.
#[derive(AsRef, Clone, Debug, Display, From, GraphQLScalar, Into)]
#[graphql(
    name = "RealtyCountry",
    with = scalar::Via::<domain::realty::Country>,
)]
pub struct Country(domain::realty::Country);

/// State of a `Realty`.
#[derive(AsRef, Clone, Debug, Display, From, GraphQLScalar, Into)]
#[graphql(
    name = "RealtyState",
    with = scalar::Via::<domain::realty::State>,
)]
pub struct State(domain::realty::State);

/// City of a `Realty`.
#[derive(AsRef, Clone, Debug, Display, From, GraphQLScalar, Into)]
#[graphql(
    name = "RealtyCity",
    with = scalar::Via::<domain::realty::City>,
)]
pub struct City(domain::realty::City);

/// Street of a `Realty`.
#[derive(AsRef, Clone, Debug, Display, From, GraphQLScalar, Into)]
#[graphql(
    name = "RealtyStreet",
    with = scalar::Via::<domain::realty::Street>,
)]
pub struct Street(domain::realty::Street);

/// Zip code of a `Realty`.
#[derive(AsRef, Clone, Debug, Display, From, GraphQLScalar, Into)]
#[graphql(
    name = "RealtyZipCode",
    with = scalar::Via::<domain::realty::ZipCode>,
)]
pub struct ZipCode(domain::realty::ZipCode);

/// Building name of a `Realty`.
#[derive(AsRef, Clone, Debug, Display, From, GraphQLScalar, Into)]
#[graphql(
    name = "RealtyBuildingName",
    with = scalar::Via::<domain::realty::BuildingName>,
)]
pub struct BuildingName(domain::realty::BuildingName);

/// Apartment number of a `Realty`.
#[derive(AsRef, Clone, Debug, Display, From, GraphQLScalar, Into)]
#[graphql(
    name = "RealtyApartmentNum",
    with = scalar::Via::<domain::realty::ApartmentNum>,
)]
pub struct ApartmentNum(domain::realty::ApartmentNum);

/// Room number of a `Realty`.
#[derive(AsRef, Clone, Debug, Display, From, GraphQLScalar, Into)]
#[graphql(
    name = "RealtyRoomNum",
    with = scalar::Via::<domain::realty::RoomNum>,
)]
pub struct RoomNum(domain::realty::RoomNum);

/// Kind of a `Realty`.
#[derive(Clone, Copy, Debug, GraphQLEnum)]
#[graphql(name = "RealtyKind")]
pub enum Kind {
    /// An apartment.
    Apartment,

    /// A building.
    Building,

    /// A room.
    Room,
}

impl From<domain::realty::Kind> for Kind {
    fn from(kind: domain::realty::Kind) -> Self {
        use domain::realty::Kind as K;
        match kind {
            K::Apartment => Self::Apartment,
            K::Building => Self::Building,
            K::Room => Self::Room,
        }
    }
}

pub mod list {
    //! Definitions related to the [`Realty`] list.

    use derive_more::{AsRef, From, Into};
    use juniper::{graphql_object, GraphQLScalar};
    use service::{query, read, Query as _};

    use super::{Id, Realty};
    use crate::{api::scalar, AsError, Context, Error};

    /// Cursor for the `Realty` list.
    #[derive(AsRef, Clone, Copy, Debug, From, GraphQLScalar, Into)]
    #[from(Id, read::realty::list::Cursor)]
    #[graphql(
        name = "RealtyListCursor",
        with = scalar::Via::<read::realty::list::Cursor>,
    )]
    pub struct Cursor(pub read::realty::list::Cursor);

    /// Edge in the [`Realty`] list.
    #[derive(Clone, Copy, Debug, From, Into)]
    pub struct Edge(read::realty::list::Edge);

    /// Edge in the `Realty` list.
    #[graphql_object(name = "RealtyListEdge", context = Context)]
    impl Edge {
        /// Cursor of this `RealtyListEdge`.
        #[must_use]
        pub fn cursor(&self) -> Cursor {
            self.0.cursor.into()
        }

        /// Node of this `RealtyListEdge`.
        #[must_use]
        pub fn node(&self) -> Realty {
            #[expect(
                unsafe_code,
                reason = "`Edge` loaded from repository guarantees `User` \
                          existence"
            )]
            unsafe {
                Realty::new_unchecked(self.0.node)
            }
        }
    }

    /// Connection of the [`Realty`] list.
    #[derive(Clone, Debug, From, Into)]
    pub struct Connection(read::realty::list::Connection);

    /// Connection of the `Realty` list.
    #[graphql_object(name = "RealtyListConnection", context = Context)]
    impl Connection {
        /// Edges of this `RealtyListConnection`.
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
        /// Underlying [`read::realty::list::PageInfo`].
        info: read::realty::list::PageInfo,

        /// Start cursor of the page.
        start_cursor: Option<Cursor>,

        /// End cursor of the page.
        end_cursor: Option<Cursor>,
    }

    /// Information about a `RealtyListConnection` page.
    #[graphql_object(name = "RealtyListPageInfo", context = Context)]
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

        /// Total `Realty` count.
        pub async fn total_count(&self, ctx: &Context) -> Result<i32, Error> {
            ctx.service()
                .execute(query::realties::TotalCount::by(()))
                .await
                .map_err(AsError::into_error)
                .map_err(ctx.error())
                .map(Into::into)
        }
    }
}
