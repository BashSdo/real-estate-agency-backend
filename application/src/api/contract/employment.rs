use std::future;

use common::{DateTime, DateTimeOf, Money};
use futures::TryFutureExt as _;
use juniper::graphql_object;
use service::{domain, query, read, Command as _};
use tokio::sync::OnceCell;

#[cfg(doc)]
use crate::api::Contract;
use crate::{api, AsError, Context, Error};

use super::{ContractValue, Description, Id, Name};

/// Employment [`Contract`].
#[derive(Clone, Debug)]
pub struct Employment {
    /// ID of this [`Contract`].
    id: Id,

    /// Underlying [`domain::contract::Employment`].
    contract: OnceCell<domain::contract::Employment>,

    /// Employer this [`Contract`] is about.
    employer: OnceCell<api::User>,
}

impl From<domain::contract::Employment> for Employment {
    fn from(contract: domain::contract::Employment) -> Self {
        Self {
            id: contract.id.into(),
            contract: OnceCell::new_with(Some(contract)),
            employer: OnceCell::new(),
        }
    }
}

impl From<read::contract::Active<domain::contract::Employment>> for Employment {
    fn from(
        read::contract::Active(c): read::contract::Active<
            domain::contract::Employment,
        >,
    ) -> Self {
        c.into()
    }
}

impl Employment {
    /// Creates a new [`Employment`] [`Contract`] with the provided ID.
    ///
    /// # Safety
    ///
    /// Caller must ensure that provided ID related to existing
    /// [`domain::contract::Employment`].
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(id: impl Into<Id>) -> Self {
        Self {
            id: id.into(),
            contract: OnceCell::new(),
            employer: OnceCell::new(),
        }
    }

    /// Returns [`domain::contract::Employment`] representing this
    /// [`Employment`] [`Contract`].
    ///
    /// # Errors
    ///
    /// Returns an error if the [`domain::contract::Employment`] does not exist.
    async fn contract(
        &self,
        ctx: &Context,
    ) -> Result<&domain::contract::Employment, Error> {
        self.contract
            .get_or_try_init(|| {
                ctx.service()
                    .execute(query::contract::ById::by(self.id.into()))
                    .map_err(AsError::into_error)
                    .map_err(ctx.error())
                    .and_then(|c| {
                        future::ready(match c {
                            Some(domain::Contract::Employment(c)) => Ok(c),
                            _ => {
                                Err(api::query::ContractError::NotExists.into())
                            }
                        })
                    })
            })
            .await
    }
}

/// Employment `Contract`.
#[graphql_object(
    name = "EmploymentContract",
    context = Context,
    impl = ContractValue,
)]
impl Employment {
    /// Unique identifier of this `Contract`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "EmploymentContract.id",
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
            gql.name = "EmploymentContract.name",
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
            gql.name = "EmploymentContract.description",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn description(
        &self,
        ctx: &Context,
    ) -> Result<Description, Error> {
        Ok(self.contract(ctx).await?.description.clone().into())
    }

    /// Employer this `Contract` is about.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "EmploymentContract.employer",
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

    /// Base salary of employer this `Contract` is about.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "EmploymentContract.baseSalary",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn base_salary(&self, ctx: &Context) -> Result<Money, Error> {
        Ok(self.contract(ctx).await?.base_salary)
    }

    /// `DateTime` when this `Contract` was created.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "EmploymentContract.createdAt",
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
            gql.name = "EmploymentContract.expiresAt",
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
            gql.name = "EmploymentContract.terminatedAt",
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
