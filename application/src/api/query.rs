//! GraphQL [`Query`]s definitions.

use common::DateTime;
use itertools::Itertools as _;
use juniper::graphql_object;
use service::{query, read, Query as _};

use crate::{api, define_error, AsError, Context, Error};

/// Root of all GraphQL queries.
#[derive(Clone, Copy, Debug)]
pub struct Query;

impl Query {
    /// Name of the [`tracing::Span`] for the queries.
    pub(crate) const SPAN_NAME: &'static str = "GraphQL query";
}

#[graphql_object(context = Context)]
impl Query {
    /// Returns the currently authenticated `User`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "myUser",
            otel.name = Self::SPAN_NAME,
        ),
    )]
    pub async fn my_user(ctx: &Context) -> Result<api::User, Error> {
        let my_id = ctx.current_session().await?.user_id;
        ctx.service()
            .execute(query::user::ById::by(my_id.into()))
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())?
            .ok_or_else(|| UserError::NotExists.into())
            .map_err(ctx.error())
            .map(Into::into)
    }

    /// Returns the `User` with the specified ID.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `USER_NOT_EXISTS` - the `User` with the specified ID does not exist;
    /// - `NOT_EMPLOYER` - the current `User` is not an employer and tries to
    ///                    access another `User`.
    #[tracing::instrument(
        skip_all,
        fields(
            id = %id,
            gql.name = "user",
            otel.name = Self::SPAN_NAME,
        ),
    )]
    pub async fn user(
        id: api::user::Id,
        ctx: &Context,
    ) -> Result<api::user::list::Edge, Error> {
        Self::users(None, Some(id.into()), None, Some(id.into()), None, ctx)
            .await?
            .edges()
            .into_iter()
            .exactly_one()
            .map_err(|_| UserError::NotExists.into())
            .map_err(ctx.error())
    }

    /// Fetches the page of `User`s.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `PAGINATION_AMBIGUOUS` - the pagination arguments are ambiguous;
    /// - `NOT_EMPLOYER` - the current `User` is not an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            after = ?after,
            before = ?before,
            first = ?first,
            gql.name = "users",
            last = ?last,
            name = ?name.as_ref().map(ToString::to_string),
            otel.name = Self::SPAN_NAME,
        ),
    )]
    pub async fn users(
        first: Option<i32>,
        after: Option<api::user::list::Cursor>,
        last: Option<i32>,
        before: Option<api::user::list::Cursor>,
        name: Option<api::user::Name>,
        ctx: &Context,
    ) -> Result<api::user::list::Connection, Error> {
        const DEFAULT_PAGE_SIZE: i32 = 10;

        let arguments = read::user::list::Arguments::new(
            first,
            after.map(Into::into),
            last,
            before.map(Into::into),
            DEFAULT_PAGE_SIZE,
        )
        .ok_or_else(|| api::PaginationError::Ambiguous.into())
        .map_err(ctx.error())?;

        let my_id = ctx.current_session().await?.user_id;
        let is_employed = ctx
            .service()
            .execute(query::contract::Employment::by(my_id.into()))
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())?
            .is_some();
        let (is_myself, is_employer) =
            if let Some(id) = arguments.exact_cursor().copied() {
                let is_myself = api::user::Id::from(id) == my_id;
                let is_employer = ctx
                    .service()
                    .execute(query::contract::Employment::by(id))
                    .await
                    .map_err(AsError::into_error)
                    .map_err(ctx.error())?
                    .is_some();
                (is_myself, is_employer)
            } else {
                (false, false)
            };
        if !is_employed && !is_myself && !is_employer {
            return Err(api::PrivilegeError::Employer.into());
        }

        ctx.service()
            .execute(query::users::List::by(read::user::list::Selector {
                arguments,
                filter: read::user::list::Filter {
                    name: name.map(Into::into),
                },
            }))
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map(Into::into)
    }

    /// Returns the `Placement` with the specified ID.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `PLACEMENT_NOT_EXISTS` - the `Placement` with the specified ID does
    ///                            not exist.
    #[tracing::instrument(
        skip_all,
        fields(
            id = %id,
            gql.name = "placement",
            otel.name = Self::SPAN_NAME,
        ),
    )]
    pub async fn placement(
        id: api::realty::Id,
        ctx: &Context,
    ) -> Result<api::placement::list::Edge, Error> {
        Self::placements(
            None,
            Some(id.into()),
            None,
            Some(id.into()),
            None,
            None,
            ctx,
        )
        .await?
        .edges()
        .into_iter()
        .exactly_one()
        .map_err(|_| PlacementError::NotExists.into())
        .map_err(ctx.error())
    }

    /// Fetches the page of `Placement`s.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `PAGINATION_AMBIGUOUS` - the pagination arguments are ambiguous.
    #[tracing::instrument(
        skip_all,
        fields(
            after = ?after,
            before = ?before,
            first = ?first,
            gql.name = "placements",
            include_rent = ?include_rent,
            include_sale = ?include_sale,
            last = ?last,
            otel.name = Self::SPAN_NAME,
        ),
    )]
    pub async fn placements(
        first: Option<i32>,
        after: Option<api::placement::list::Cursor>,
        last: Option<i32>,
        before: Option<api::placement::list::Cursor>,
        include_sale: Option<bool>,
        include_rent: Option<bool>,
        ctx: &Context,
    ) -> Result<api::placement::list::Connection, Error> {
        const DEFAULT_PAGE_SIZE: i32 = 10;

        ctx.service()
            .execute(query::placements::List::by(
                read::placement::list::Selector {
                    arguments: read::placement::list::Arguments::new(
                        first,
                        after.map(Into::into),
                        last,
                        before.map(Into::into),
                        DEFAULT_PAGE_SIZE,
                    )
                    .ok_or_else(|| api::PaginationError::Ambiguous.into())
                    .map_err(ctx.error())?,
                    filter: read::placement::list::Filter {
                        rent: include_rent.unwrap_or(true),
                        sale: include_sale.unwrap_or(true),
                    },
                },
            ))
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map(Into::into)
    }

    /// Returns the `Contract` with the specified ID.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `CONTRACT_NOT_EXISTS` - the `Contract` with the specified ID does not
    ///                           exist;
    /// - `NOT_EMPLOYER` - the current `User` is not an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            id = %id,
            gql.name = "contract",
            otel.name = Self::SPAN_NAME,
        ),
    )]
    pub async fn contract(
        id: api::contract::Id,
        ctx: &Context,
    ) -> Result<api::contract::list::Edge, Error> {
        let my_id = ctx.current_session().await?.user_id;
        let is_employed = ctx
            .service()
            .execute(query::contract::Employment::by(my_id.into()))
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())?
            .is_some();
        if !is_employed {
            return Err(api::PrivilegeError::Employer.into());
        }

        Self::contracts(None, Some(id.into()), None, Some(id.into()), None, ctx)
            .await?
            .edges()
            .into_iter()
            .exactly_one()
            .map_err(|_| ContractError::NotExists.into())
            .map_err(ctx.error())
    }

    /// Fetches the page of `Contract`s.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `PAGINATION_AMBIGUOUS` - the pagination arguments are ambiguous;
    /// - `NOT_EMPLOYER` - the current `User` is not an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            after = ?after,
            before = ?before,
            first = ?first,
            gql.name = "contracts",
            last = ?last,
            name = ?name.as_ref().map(ToString::to_string),
            otel.name = Self::SPAN_NAME,
        ),
    )]
    pub async fn contracts(
        first: Option<i32>,
        after: Option<api::contract::list::Cursor>,
        last: Option<i32>,
        before: Option<api::contract::list::Cursor>,
        name: Option<api::contract::Name>,
        ctx: &Context,
    ) -> Result<api::contract::list::Connection, Error> {
        const DEFAULT_PAGE_SIZE: i32 = 10;

        let my_id = ctx.current_session().await?.user_id;
        let is_employed = ctx
            .service()
            .execute(query::contract::Employment::by(my_id.into()))
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())?
            .is_some();
        if !is_employed {
            return Err(api::PrivilegeError::Employer.into());
        }

        ctx.service()
            .execute(query::contracts::List::by(
                read::contract::list::Selector {
                    arguments: read::contract::list::Arguments::new(
                        first,
                        after.map(Into::into),
                        last,
                        before.map(Into::into),
                        DEFAULT_PAGE_SIZE,
                    )
                    .ok_or_else(|| api::PaginationError::Ambiguous.into())
                    .map_err(ctx.error())?,
                    filter: read::contract::list::Filter {
                        name: name.map(Into::into),
                    },
                },
            ))
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map(Into::into)
    }

    /// Returns the `Realty` with the specified ID.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `REALTY_NOT_EXISTS` - the `Realty` with the specified ID does not
    ///                         exist;
    /// - `NOT_EMPLOYER` - the current `User` is not an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            id = %id,
            gql.name = "realty",
            otel.name = Self::SPAN_NAME,
        ),
    )]
    pub async fn realty(
        id: api::realty::Id,
        ctx: &Context,
    ) -> Result<api::realty::list::Edge, Error> {
        let my_id = ctx.current_session().await?.user_id;
        let is_employed = ctx
            .service()
            .execute(query::contract::Employment::by(my_id.into()))
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())?
            .is_some();
        if !is_employed {
            return Err(api::PrivilegeError::Employer.into());
        }

        Self::realties(None, Some(id.into()), None, Some(id.into()), None, ctx)
            .await?
            .edges()
            .into_iter()
            .exactly_one()
            .map_err(|_| RealtyError::NotExists.into())
            .map_err(ctx.error())
    }

    /// Fetches the page of `Realty`s.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `PAGINATION_AMBIGUOUS` - the pagination arguments are ambiguous;
    /// - `NOT_EMPLOYER` - the current `User` is not an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            address = ?address.as_ref().map(ToString::to_string),
            after = ?after,
            before = ?before,
            first = ?first,
            gql.name = "realties",
            last = ?last,
            otel.name = Self::SPAN_NAME,
        ),
    )]
    pub async fn realties(
        first: Option<i32>,
        after: Option<api::realty::list::Cursor>,
        last: Option<i32>,
        before: Option<api::realty::list::Cursor>,
        address: Option<api::realty::Address>,
        ctx: &Context,
    ) -> Result<api::realty::list::Connection, Error> {
        const DEFAULT_PAGE_SIZE: i32 = 10;

        let my_id = ctx.current_session().await?.user_id;
        let is_employed = ctx
            .service()
            .execute(query::contract::Employment::by(my_id.into()))
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())?
            .is_some();
        if !is_employed {
            return Err(api::PrivilegeError::Employer.into());
        }

        ctx.service()
            .execute(query::realties::List::by(read::realty::list::Selector {
                arguments: read::realty::list::Arguments::new(
                    first,
                    after.map(Into::into),
                    last,
                    before.map(Into::into),
                    DEFAULT_PAGE_SIZE,
                )
                .ok_or_else(|| api::PaginationError::Ambiguous.into())
                .map_err(ctx.error())?,
                filter: read::realty::list::Filter {
                    address: address.map(Into::into),
                },
            }))
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map(Into::into)
    }

    /// Calculates the `SalaryReport` for the specified period.
    #[tracing::instrument(
        skip_all,
        fields(
            end_at = ?end_at,
            gql.name = "salaryReport",
            otel.name = Self::SPAN_NAME,
            start_at = ?start_at,
        ),
    )]
    pub async fn salary_report(
        start_at: DateTime,
        end_at: DateTime,
        ctx: &Context,
    ) -> Result<api::report::Salary, Error> {
        let my_id = ctx.current_session().await?.user_id;
        let is_employed = ctx
            .service()
            .execute(query::contract::Employment::by(my_id.into()))
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())?
            .is_some();
        if !is_employed {
            return Err(api::PrivilegeError::Employer.into());
        }

        ctx.service()
            .execute(query::report::Salary {
                start: start_at,
                end: end_at,
            })
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map(Into::into)
    }
}

define_error! {
    enum ContractError {
        #[code = "CONTRACT_NOT_EXISTS"]
        #[status = NOT_FOUND]
        #[message = "`Contract` with the specified ID does not exist"]
        NotExists,
    }
}

define_error! {
    enum PlacementError {
        #[code = "PLACEMENT_NOT_EXISTS"]
        #[status = NOT_FOUND]
        #[message = "`Placement` with the specified ID does not exist"]
        NotExists,
    }
}

define_error! {
    enum RealtyError {
        #[code = "REALTY_NOT_EXISTS"]
        #[status = NOT_FOUND]
        #[message = "`Realty` with the specified ID does not exist"]
        NotExists,
    }
}

define_error! {
    enum UserError {
        #[code = "USER_NOT_EXISTS"]
        #[status = NOT_FOUND]
        #[message = "`User` with the specified ID does not exist"]
        NotExists,
    }
}
