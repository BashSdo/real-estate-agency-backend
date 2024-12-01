use std::future;

use common::{DateTime, DateTimeOf, Money, Percent};
use futures::TryFutureExt as _;
use juniper::graphql_object;
use service::{domain, query, read, Command as _};
use tokio::sync::OnceCell;

#[cfg(doc)]
use crate::api::{Contract, Realty, User};
use crate::{api, AsError, Context, Error};

use super::{ContractValue, Description, Id, Name};

/// [`Contract`] about managing a [`Realty`] for rent.
#[derive(Clone, Debug)]
pub struct ManagementForRent {
    /// ID of this [`Contract`].
    id: Id,

    /// Underlying [`domain::contract::ManagementForRent`].
    contract: OnceCell<domain::contract::ManagementForRent>,

    /// [`Realty`] this [`Contract`] is about.
    realty: OnceCell<api::Realty>,

    /// [`User`] who is landlord of the [`Realty`] this [`Contract`] is about.
    landlord: OnceCell<api::User>,

    /// [`User`] who is employer signing this [`Contract`].
    employer: OnceCell<api::User>,
}

impl From<domain::contract::ManagementForRent> for ManagementForRent {
    fn from(contract: domain::contract::ManagementForRent) -> Self {
        Self {
            id: contract.id.into(),
            contract: OnceCell::new_with(Some(contract)),
            realty: OnceCell::new(),
            employer: OnceCell::new(),
            landlord: OnceCell::new(),
        }
    }
}

impl From<read::contract::Active<domain::contract::ManagementForRent>>
    for ManagementForRent
{
    fn from(
        read::contract::Active(c): read::contract::Active<
            domain::contract::ManagementForRent,
        >,
    ) -> Self {
        c.into()
    }
}

impl ManagementForRent {
    /// Creates a new [`ManagementForRent`] [`Contract`] with the provided ID.
    ///
    /// # Safety
    ///
    /// Caller must ensure that provided ID related to existing
    /// [`domain::contract::ManagementForRent`], otherwise accessing this
    /// [`Contract`] will result with an error.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(id: impl Into<Id>) -> Self {
        Self {
            id: id.into(),
            contract: OnceCell::new(),
            realty: OnceCell::new(),
            employer: OnceCell::new(),
            landlord: OnceCell::new(),
        }
    }

    /// Returns [`domain::contract::ManagementForRent`] representing this
    /// [`ManagementForRent`] [`Contract`].
    ///
    /// # Errors
    ///
    /// Returns an error if the [`domain::contract::ManagementForRent`] does
    /// not exist.
    async fn contract(
        &self,
        ctx: &Context,
    ) -> Result<&domain::contract::ManagementForRent, Error> {
        self.contract
            .get_or_try_init(|| {
                ctx.service()
                    .execute(query::contract::ById::by(self.id.into()))
                    .map_err(AsError::into_error)
                    .map_err(ctx.error())
                    .and_then(|c| {
                        future::ready(match c {
                            Some(domain::Contract::ManagementForRent(c)) => {
                                Ok(c)
                            }
                            _ => {
                                Err(api::query::ContractError::NotExists.into())
                            }
                        })
                    })
            })
            .await
    }
}

/// `Contract` about managing a `Realty` for rent.
#[graphql_object(
    name = "ManagementForRentContract",
    context = Context,
    impl = ContractValue,
)]
impl ManagementForRent {
    /// Unique identifier of this `Contract`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "ManagementForRentContract.id",
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
            gql.name = "ManagementForRentContract.name",
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
            gql.name = "ManagementForRentContract.description",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn description(
        &self,
        ctx: &Context,
    ) -> Result<Description, Error> {
        Ok(self.contract(ctx).await?.description.clone().into())
    }

    /// `Realty` this `Contract` is about.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "ManagementForRentContract.realty",
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

    /// `User` who is the landlord of the `Realty` this `Contract` is about.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "ManagementForRentContract.landlord",
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

    /// `User` who is the employer signing this `Contract`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "ManagementForRentContract.employer",
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

    /// Expected rent price of the `Realty` this `Contract` is about.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "ManagementForRentContract.expectedPrice",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn expected_price(&self, ctx: &Context) -> Result<Money, Error> {
        Ok(self.contract(ctx).await?.expected_price)
    }

    /// Expected deposit of the `Realty` this `Contract` is about.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "ManagementForRentContract.expectedDeposit",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn expected_deposit(
        &self,
        ctx: &Context,
    ) -> Result<Option<Money>, Error> {
        Ok(self.contract(ctx).await?.expected_deposit)
    }

    /// One-time fee the landlord charges for managing the `Realty`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "ManagementForRentContract.oneTimeFee",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn one_time_fee(
        &self,
        ctx: &Context,
    ) -> Result<Option<Money>, Error> {
        Ok(self.contract(ctx).await?.one_time_fee)
    }

    /// Monthly fee the landlord charges for managing the `Realty`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "ManagementForRentContract.monthlyFee",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn monthly_fee(
        &self,
        ctx: &Context,
    ) -> Result<Option<Money>, Error> {
        Ok(self.contract(ctx).await?.monthly_fee)
    }

    /// Percentage fee from rent the landlord charges for managing the `Realty`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "ManagementForRentContract.percentFee",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn percent_fee(
        &self,
        ctx: &Context,
    ) -> Result<Option<Percent>, Error> {
        Ok(self.contract(ctx).await?.percent_fee)
    }

    /// Indicator whether this `Contract` is placed.
    ///
    /// Placed `Contract`s are visible as `Placement`s.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "ManagementForRentContract.isPlaced",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn is_placed(&self, ctx: &Context) -> Result<bool, Error> {
        Ok(self.contract(ctx).await?.is_placed)
    }

    /// `DateTime` when this `Contract` was created.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "ManagementForRentContract.createdAt",
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
            gql.name = "ManagementForRentContract.expiresAt",
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
            gql.name = "ManagementForRentContract.terminatedAt",
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