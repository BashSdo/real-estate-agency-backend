use std::future;

use common::{DateTime, DateTimeOf, Money};
use futures::TryFutureExt as _;
use juniper::graphql_object;
use service::{domain, query, read, Command as _};
use tokio::sync::OnceCell;

#[cfg(doc)]
use crate::api::{Contract, Realty, User};
use crate::{api, AsError, Context, Error};

use super::{ContractValue, Description, Id, Name};

/// [`Contract`] about some [`Realty`] being sold.
#[derive(Clone, Debug)]
pub struct Sale {
    /// ID of this [`Contract`].
    id: Id,

    /// Underlying [`domain::contract::Sale`].
    contract: OnceCell<domain::contract::Sale>,

    /// Realty this [`Contract`] is about.
    realty: OnceCell<api::Realty>,

    /// [`User`] who is purchasing the [`Realty`].
    purchaser: OnceCell<api::User>,

    /// [`User`] who is selling the [`Realty`].
    landlord: OnceCell<api::User>,

    /// [`User`] who is the employer of the [`User`] selling the [`Realty`].
    employer: OnceCell<api::User>,
}

impl From<domain::contract::Sale> for Sale {
    fn from(contract: domain::contract::Sale) -> Self {
        Self {
            id: contract.id.into(),
            contract: OnceCell::new_with(Some(contract)),
            realty: OnceCell::new(),
            purchaser: OnceCell::new(),
            landlord: OnceCell::new(),
            employer: OnceCell::new(),
        }
    }
}

impl From<read::contract::Active<domain::contract::Sale>> for Sale {
    fn from(
        read::contract::Active(c): read::contract::Active<
            domain::contract::Sale,
        >,
    ) -> Self {
        c.into()
    }
}

impl Sale {
    /// Creates a new [`Sale`] [`Contract`] with the provided ID.
    ///
    /// # Safety
    ///
    /// Caller must ensure that provided ID related to existing
    /// [`domain::contract::Sale`].
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(id: impl Into<Id>) -> Self {
        Self {
            id: id.into(),
            contract: OnceCell::new(),
            realty: OnceCell::new(),
            purchaser: OnceCell::new(),
            landlord: OnceCell::new(),
            employer: OnceCell::new(),
        }
    }

    /// Returns [`domain::contract::Sale`] representing this [`Sale`]
    /// [`Contract`].
    ///
    /// # Errors
    ///
    /// Returns an error if the [`domain::contract::Sale`] does not exist.
    async fn contract(
        &self,
        ctx: &Context,
    ) -> Result<&domain::contract::Sale, Error> {
        self.contract
            .get_or_try_init(|| {
                ctx.service()
                    .execute(query::contract::ById::by(self.id.into()))
                    .map_err(AsError::into_error)
                    .map_err(ctx.error())
                    .and_then(|c| {
                        future::ready(match c {
                            Some(domain::Contract::Sale(c)) => Ok(c),
                            _ => {
                                Err(api::query::ContractError::NotExists.into())
                            }
                        })
                    })
            })
            .await
    }
}

/// `Contract` about some `Realty` being sold.
#[graphql_object(
    name = "SaleContract",
    context = Context,
    impl = ContractValue,
)]
impl Sale {
    /// Unique identifier of this `Contract`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "SaleContract.id",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub fn id(&self) -> Id {
        self.id
    }

    /// Name of this `Contract`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "SaleContract.name",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn name(&self, ctx: &Context) -> Result<Name, Error> {
        Ok(self.contract(ctx).await?.name.clone().into())
    }

    /// Description of this `Contract`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "SaleContract.description",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn description(
        &self,
        ctx: &Context,
    ) -> Result<Description, Error> {
        Ok(self.contract(ctx).await?.description.clone().into())
    }

    /// `Realty` being sold.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "SaleContract.realty",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn realty(&self, ctx: &Context) -> Result<&api::Realty, Error> {
        let id = self.contract(ctx).await?.realty_id;
        self.realty
            .get_or_try_init(|| {
                ctx.service()
                    .execute(query::realty::ById::by(id))
                    .map_err(AsError::into_error)
                    .map_err(ctx.error())
                    .and_then(|u| {
                        future::ready(u.map_or_else(
                            || Err(api::query::RealtyError::NotExists.into()),
                            |u| Ok(u.into()),
                        ))
                    })
            })
            .await
    }

    /// Purchaser of this `Contract`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "SaleContract.purchaser",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn purchaser(&self, ctx: &Context) -> Result<&api::User, Error> {
        let id = self.contract(ctx).await?.purchaser_id;
        self.purchaser
            .get_or_try_init(|| {
                ctx.service()
                    .execute(query::user::ById::by(id))
                    .map_err(AsError::into_error)
                    .map_err(ctx.error())
                    .and_then(|u| {
                        future::ready(u.map_or_else(
                            || Err(api::query::UserError::NotExists.into()),
                            |u| Ok(u.into()),
                        ))
                    })
            })
            .await
    }

    /// Landlord of this `Contract`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "SaleContract.landlord",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn landlord(&self, ctx: &Context) -> Result<&api::User, Error> {
        let id = self.contract(ctx).await?.landlord_id;
        self.landlord
            .get_or_try_init(|| {
                ctx.service()
                    .execute(query::user::ById::by(id))
                    .map_err(AsError::into_error)
                    .map_err(ctx.error())
                    .and_then(|u| {
                        future::ready(u.map_or_else(
                            || Err(api::query::UserError::NotExists.into()),
                            |u| Ok(u.into()),
                        ))
                    })
            })
            .await
    }

    /// Employer of this `Contract`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "SaleContract.employer",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn employer(&self, ctx: &Context) -> Result<&api::User, Error> {
        let id = self.contract(ctx).await?.employer_id;
        self.employer
            .get_or_try_init(|| {
                ctx.service()
                    .execute(query::user::ById::by(id))
                    .map_err(AsError::into_error)
                    .map_err(ctx.error())
                    .and_then(|u| {
                        future::ready(u.map_or_else(
                            || Err(api::query::UserError::NotExists.into()),
                            |u| Ok(u.into()),
                        ))
                    })
            })
            .await
    }

    /// Price of the `Realty` this `Contract` is about.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "SaleContract.price",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn price(&self, ctx: &Context) -> Result<Money, Error> {
        Ok(self.contract(ctx).await?.price)
    }

    /// Deposit the purchaser was paid before this `Contract` was signed.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "SaleContract.deposit",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn deposit(&self, ctx: &Context) -> Result<Option<Money>, Error> {
        Ok(self.contract(ctx).await?.deposit)
    }

    /// `DateTime` when this `Contract` was created.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "SaleContract.createdAt",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn created_at(&self, ctx: &Context) -> Result<DateTime, Error> {
        Ok(self.contract(ctx).await?.created_at.coerce())
    }

    /// `DateTime` when this `Contract` expires.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "SaleContract.expiresAt",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn expires_at(
        &self,
        ctx: &Context,
    ) -> Result<Option<DateTime>, Error> {
        Ok(self.contract(ctx).await?.expires_at.map(DateTimeOf::coerce))
    }

    /// `DateTime` when this `Contract` was terminated.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "SaleContract.terminatedAt",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn terminated_at(
        &self,
        ctx: &Context,
    ) -> Result<Option<DateTime>, Error> {
        Ok(self
            .contract(ctx)
            .await?
            .terminated_at
            .map(DateTimeOf::coerce))
    }
}
