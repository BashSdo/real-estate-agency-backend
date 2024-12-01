//! [`Placement`]-related definitions.

use common::{DateTime, Handler as _, Money};
use futures::{
    future, stream::FuturesUnordered, TryFutureExt as _, TryStreamExt as _,
};
use itertools::Either;
use juniper::{graphql_object, GraphQLObject};
use service::{query, read};
use tokio::sync::OnceCell;

#[cfg(doc)]
use crate::api::{Contract, Realty};
use crate::{api, AsError, Context, Error};

/// Placement of some [`Realty`].
#[derive(Debug)]
pub struct Placement {
    /// Underlying [`read::Placement`].
    placement: read::Placement,

    /// [`Contract`] for renting the [`Realty`].
    management_for_rent_contract:
        OnceCell<Option<api::contract::ManagementForRent>>,

    /// [`Contract`] for selling the [`Realty`].
    management_for_sale_contract:
        OnceCell<Option<api::contract::ManagementForSale>>,

    /// [`Realty`] this [`Placement`] is about.
    realty: OnceCell<api::Realty>,
}

impl From<read::Placement> for Placement {
    fn from(placement: read::Placement) -> Self {
        Self {
            placement,
            management_for_rent_contract: OnceCell::new(),
            management_for_sale_contract: OnceCell::new(),
            realty: OnceCell::new(),
        }
    }
}

impl Placement {
    /// Returns [`Contract`] for renting the [`Realty`] this [`Placement`] is
    /// about.
    ///
    /// # Errors
    ///
    /// Errors if [`Contract`] doesn't exist.
    async fn management_for_rent_contract(
        &self,
        ctx: &Context,
    ) -> Result<Option<&api::contract::ManagementForRent>, Error> {
        Ok(self
            .management_for_rent_contract
            .get_or_try_init(|| async {
                ctx.service()
                    .execute(query::contract::ManagementForRent::by(
                        self.placement.realty_id,
                    ))
                    .await
                    .map_err(AsError::into_error)
                    .map_err(ctx.error())
                    .map(|c| c.map(Into::into))
            })
            .await?
            .as_ref())
    }

    /// Returns [`Contract`] for selling the [`Realty`] this [`Placement`] is
    /// about.
    ///
    /// # Errors
    ///
    /// Errors if [`Contract`] doesn't exist.
    async fn management_for_sale_contract(
        &self,
        ctx: &Context,
    ) -> Result<Option<&api::contract::ManagementForSale>, Error> {
        Ok(self
            .management_for_sale_contract
            .get_or_try_init(|| async {
                ctx.service()
                    .execute(query::contract::ManagementForSale::by(
                        self.placement.realty_id,
                    ))
                    .await
                    .map_err(AsError::into_error)
                    .map_err(ctx.error())
                    .map(|c| c.map(Into::into))
            })
            .await?
            .as_ref())
    }
}

/// Placement of some `Realty`.
#[graphql_object(context = Context)]
impl Placement {
    /// `Realty` this `Placement` is about.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "Placement.realty",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn realty(&self, ctx: &Context) -> Result<&api::Realty, Error> {
        self.realty
            .get_or_try_init(|| async {
                Ok(ctx
                    .service()
                    .execute(query::realty::ById::by(self.placement.realty_id))
                    .await
                    .map_err(AsError::into_error)
                    .map_err(ctx.error())?
                    .expect("`Placement.realty` should exists")
                    .into())
            })
            .await
    }

    /// `Contract` for renting the `Realty` this `Placement` is about.
    ///
    /// No `Contract` is returned if the `Realty` is not for rent, or
    /// the current `User` is not an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "Placement.rentContract",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn rent_contract(
        &self,
        ctx: &Context,
    ) -> Result<Option<&api::contract::ManagementForRent>, Error> {
        let Some(session) = ctx.try_current_session().await? else {
            return Ok(None);
        };

        let is_employed = ctx
            .service()
            .execute(query::contract::Employment::by(session.user_id.into()))
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())?
            .is_some();
        if !is_employed {
            return Ok(None);
        }

        self.management_for_rent_contract(ctx).await
    }

    /// `Contract` for selling the `Realty` this `Placement` is about.
    ///
    /// No `Contract` is returned if the `Realty` is not for sale, or
    /// the current `User` is not an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "Placement.saleContract",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn sale_contract(
        &self,
        ctx: &Context,
    ) -> Result<Option<&api::contract::ManagementForSale>, Error> {
        let Some(session) = ctx.try_current_session().await? else {
            return Ok(None);
        };

        let is_employed = ctx
            .service()
            .execute(query::contract::Employment::by(session.user_id.into()))
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())?
            .is_some();
        if !is_employed {
            return Ok(None);
        }

        self.management_for_sale_contract(ctx).await
    }

    /// Returns rent information for the `Realty` this `Placement` is about.
    ///
    /// No information is returned if the `Realty` is not for rent.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "Placement.rentInfo",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn rent_info(
        &self,
        ctx: &Context,
    ) -> Result<Option<RentInfo>, Error> {
        let Some(c) = self.management_for_rent_contract(ctx).await? else {
            return Ok(None);
        };

        Ok(Some(RentInfo {
            price: c.expected_price(ctx).await?,
            deposit: c.expected_deposit(ctx).await?,
            employer: c.employer(ctx).await?.clone(),
        }))
    }

    /// Returns sale information for the `Realty` this `Placement` is about.
    ///
    /// No information is returned if the `Realty` is not for sale.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "Placement.saleInfo",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn sale_info(
        &self,
        ctx: &Context,
    ) -> Result<Option<SaleInfo>, Error> {
        let Some(c) = self.management_for_sale_contract(ctx).await? else {
            return Ok(None);
        };

        Ok(Some(SaleInfo {
            price: c.expected_price(ctx).await?,
            deposit: c.expected_deposit(ctx).await?,
            employer: c.employer(ctx).await?.clone(),
        }))
    }

    /// `DateTime` when the `Realty` was placed.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "Placement.placedAt",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn placed_at(&self, ctx: &Context) -> Result<DateTime, Error> {
        [
            Either::Left(self.management_for_rent_contract(ctx).and_then(
                |c| async move {
                    Ok(match c {
                        Some(c) => Some(c.created_at(ctx).await?),
                        None => None,
                    })
                },
            )),
            Either::Right(self.management_for_sale_contract(ctx).and_then(
                |c| async move {
                    Ok(match c {
                        Some(c) => Some(c.created_at(ctx).await?),
                        None => None,
                    })
                },
            )),
        ]
        .into_iter()
        .collect::<FuturesUnordered<_>>()
        .try_filter_map(future::ok)
        .try_collect::<Vec<_>>()
        .await?
        .into_iter()
        .min()
        .ok_or_else(|| api::query::ContractError::NotExists.into())
    }
}

/// Information about `Realty` rent.
#[derive(Clone, Debug, GraphQLObject)]
#[graphql(name = "PlacementRentInfo", context = Context)]
pub struct RentInfo {
    /// Price of the rent.
    pub price: Money,

    /// Deposit the purchaser should pay.
    pub deposit: Option<Money>,

    /// Employer managing the `Realty`.
    pub employer: api::User,
}

/// Information about `Realty` sale.
#[derive(Clone, Debug, GraphQLObject)]
#[graphql(name = "PlacementSaleInfo", context = Context)]
pub struct SaleInfo {
    /// Price of the `Realty`.
    pub price: Money,

    /// Deposit the purchaser should pay.
    pub deposit: Option<Money>,

    /// Employer managing the `Realty`.
    pub employer: api::User,
}

pub mod list {
    //! Definitions related to a [`Placement`] list.

    use derive_more::{AsRef, From, Into};
    use juniper::{graphql_object, GraphQLScalar};
    use service::{query, read, Query as _};

    use crate::{
        api::{self, scalar},
        AsError, Context, Error,
    };

    use super::Placement;

    /// Cursor for the `Placement` list.
    #[derive(AsRef, Clone, Copy, Debug, From, GraphQLScalar, Into)]
    #[from(api::realty::Id, read::placement::list::Cursor)]
    #[graphql(
        name = "PlacementListCursor",
        with = scalar::Via::<read::placement::list::Cursor>,
    )]
    pub struct Cursor(pub read::placement::list::Cursor);

    /// Edge in the [`Placement`] list.
    #[derive(Clone, Copy, Debug, From, Into)]
    pub struct Edge(read::placement::list::Edge);

    /// Edge in the `Placement` list.
    #[graphql_object(name = "PlacementListEdge", context = Context)]
    impl Edge {
        /// Cursor of this `PlacementListEdge`.
        #[must_use]
        pub fn cursor(&self) -> Cursor {
            self.0.cursor.into()
        }

        /// Node of this `PlacementListEdge`.
        #[must_use]
        pub fn node(&self) -> Placement {
            self.0.node.into()
        }
    }

    /// Connection of the [`Placement`] list.
    #[derive(Clone, Debug, From, Into)]
    pub struct Connection(read::placement::list::Connection);

    /// Connection of the `Contract` list.
    #[graphql_object(name = "PlacementListConnection", context = Context)]
    impl Connection {
        /// Edges in this `PlacementListConnection`.
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
        /// Underlying [`read::placement::list::PageInfo`].
        info: read::placement::list::PageInfo,

        /// Start cursor of the page.
        start_cursor: Option<Cursor>,

        /// End cursor of the page.
        end_cursor: Option<Cursor>,
    }

    /// Information about a `PlacementListConnection` page.
    #[graphql_object(name = "PlacementListPageInfo", context = Context)]
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

        /// Total `Placement`s count.
        pub async fn total_count(&self, ctx: &Context) -> Result<i32, Error> {
            ctx.service()
                .execute(query::placements::TotalCount::by(()))
                .await
                .map_err(AsError::into_error)
                .map_err(ctx.error())
                .map(Into::into)
        }
    }
}
