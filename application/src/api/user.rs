//! [`User`]-related definitions.

use common::DateTime;
use derive_more::{AsRef, Display, From, Into};
use futures::{
    future::{self, Either},
    TryFutureExt as _,
};
use juniper::{graphql_object, GraphQLScalar};
use service::{domain, query, Query};
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::{
    api::{self, scalar},
    AsError, Context, Error,
};

/// A [`User`] of the system.
#[derive(Clone, Debug, From)]
pub struct User {
    /// ID of this [`User`].
    pub id: Id,

    /// [`domain::User`] representing this [`User`].
    user: OnceCell<domain::User>,
}

impl From<domain::User> for User {
    fn from(user: domain::User) -> Self {
        Self {
            id: user.id.into(),
            user: OnceCell::new_with(Some(user)),
        }
    }
}

impl User {
    /// Creates a new [`User`] with the provided ID.
    ///
    /// # Safety
    ///
    /// Caller must ensure that [`User`] with the provided ID exists,
    /// otherwise accessing this [`User`] will result with an error.
    #[expect(unsafe_code, reason = "bypass")]
    #[must_use]
    pub unsafe fn new_unchecked(id: impl Into<Id>) -> Self {
        Self {
            id: id.into(),
            user: OnceCell::new(),
        }
    }

    /// Returns the [`domain::User`] representing this [`User`].
    ///
    /// # Errors
    ///
    /// Error if the [`domain::User`] doesn't exist.
    async fn user(&self, ctx: &Context) -> Result<&domain::User, Error> {
        let id = self.id.into();
        self.user
            .get_or_try_init(|| {
                ctx.service()
                    .execute(query::user::ById::by(id))
                    .map_err(AsError::into_error)
                    .map_err(ctx.error())
                    .and_then(|u| {
                        future::ready(u.ok_or_else(|| {
                            api::query::UserError::NotExists.into()
                        }))
                    })
            })
            .await
    }
}

/// A `User` of the system.
#[graphql_object(context = Context)]
impl User {
    /// Unique identifier of this `User`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "User.id",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub fn id(&self) -> Id {
        self.id
    }

    /// Name of this `User`.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "User.name",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn name(&self, ctx: &Context) -> Result<Name, Error> {
        Ok(self.user(ctx).await?.name.clone().into())
    }

    /// Login of this `User`.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `NOT_EMPLOYER` - if the current `User` is not an employer, not this
    ///                    `User`, and this `User` is not an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "User.login",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn login(&self, ctx: &Context) -> Result<Option<Login>, Error> {
        let my_id = ctx.try_current_session().await?.map(|s| s.user_id);

        let is_current = Some(self.id) == my_id;
        let is_employed = if let Some(my_id) = my_id {
            Either::Left(
                ctx.service()
                    .execute(query::contract::Employment::by(my_id.into()))
                    .map_err(AsError::into_error)
                    .map_err(ctx.error())
                    .map_ok(|c| c.is_some()),
            )
        } else {
            Either::Right(future::ok(false))
        };

        Ok(if is_current || is_employed.await? {
            Some(self.user(ctx).await?.login.clone().into())
        } else {
            None
        })
    }

    /// Email of this `User`.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `NOT_EMPLOYER` - if the current `User` is not an employer, not this
    ///                    `User`, and this `User` is not an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "User.email",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn email(&self, ctx: &Context) -> Result<Option<Email>, Error> {
        let my_id = ctx.try_current_session().await?.map(|s| s.user_id);

        let is_current = Some(self.id) == my_id;
        let is_employer = ctx
            .service()
            .execute(query::contract::Employment::by(self.id.into()))
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map_ok(|c| c.is_some());
        let is_employed = if let Some(my_id) = my_id {
            Either::Left(
                ctx.service()
                    .execute(query::contract::Employment::by(my_id.into()))
                    .map_err(AsError::into_error)
                    .map_err(ctx.error())
                    .map_ok(|c| c.is_some()),
            )
        } else {
            Either::Right(future::ok(false))
        };

        Ok(if is_current || is_employer.await? || is_employed.await? {
            self.user(ctx).await?.email.clone().map(Into::into)
        } else {
            None
        })
    }

    /// Phone of this `User`.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `NOT_EMPLOYER` - if the current `User` is not an employer, not this
    ///                    `User`, and this `User` is not an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "User.phone",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn phone(&self, ctx: &Context) -> Result<Option<Phone>, Error> {
        let my_id = ctx.try_current_session().await?.map(|s| s.user_id);

        let is_current = Some(self.id) == my_id;
        let is_employer = ctx
            .service()
            .execute(query::contract::Employment::by(self.id.into()))
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map_ok(|c| c.is_some());
        let is_employed = if let Some(my_id) = my_id {
            Either::Left(
                ctx.service()
                    .execute(query::contract::Employment::by(my_id.into()))
                    .map_err(AsError::into_error)
                    .map_err(ctx.error())
                    .map_ok(|c| c.is_some()),
            )
        } else {
            Either::Right(future::ok(false))
        };

        Ok(if is_current || is_employer.await? || is_employed.await? {
            self.user(ctx).await?.phone.clone().map(Into::into)
        } else {
            None
        })
    }

    /// Indicator whether this `User` is an employer.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "User.isEmployer",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn is_employer(&self, ctx: &Context) -> Result<bool, Error> {
        ctx.service()
            .execute(query::contract::Employment::by(self.id.into()))
            .await
            .map_err(AsError::into_error)
            .map_err(ctx.error())
            .map(|c| c.is_some())
    }

    /// `DateTime` when this `User` was created.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "User.createdAt",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    pub async fn created_at(&self, ctx: &Context) -> Result<DateTime, Error> {
        Ok(self.user(ctx).await?.created_at.coerce())
    }
}

/// Unique identifier of a `User`.
#[derive(
    Clone, Copy, Debug, Display, Eq, From, GraphQLScalar, Into, PartialEq,
)]
#[from(domain::user::Id)]
#[into(domain::user::Id)]
#[graphql(name = "UserId", transparent)]
pub struct Id(Uuid);

/// Name of a `User`.
#[derive(AsRef, Clone, Debug, Display, From, GraphQLScalar, Into)]
#[graphql(
    name = "UserName",
    with = scalar::Via::<domain::user::Name>,
)]
pub struct Name(domain::user::Name);

/// Login of a `User`.
#[derive(AsRef, Clone, Debug, Display, From, GraphQLScalar, Into)]
#[graphql(
    name = "UserLogin",
    with = scalar::Via::<domain::user::Login>,
)]
pub struct Login(domain::user::Login);

/// Password of a `User`.
#[derive(AsRef, Clone, Debug, From, GraphQLScalar, Into)]
#[graphql(
    name = "UserPassword",
    with = scalar::Via::<domain::user::Password>,
)]
pub struct Password(domain::user::Password);

/// Email of a `User`.
#[derive(AsRef, Clone, Debug, Display, From, GraphQLScalar, Into)]
#[graphql(
    name = "UserEmail",
    with = scalar::Via::<domain::user::Email>,
)]
pub struct Email(domain::user::Email);

/// Phone of a `User`.
#[derive(AsRef, Clone, Debug, Display, From, GraphQLScalar, Into)]
#[graphql(
    name = "UserPhone",
    with = scalar::Via::<domain::user::Phone>,
)]
pub struct Phone(domain::user::Phone);

pub mod session {
    //! [`Session`]-related definitions.
    //!
    //! [`Session`]: crate::Session

    use common::DateTime;
    use derive_more::{AsRef, From, Into};
    use juniper::{GraphQLObject, GraphQLScalar};
    use service::{command, domain};

    use crate::{
        api::{self, scalar},
        Context,
    };

    /// `Session` access token.
    #[derive(AsRef, Clone, Debug, From, GraphQLScalar, Into)]
    #[graphql(
        name = "UserAuthToken",
        with = scalar::Via::<domain::user::session::Token>,
    )]
    pub struct Token(domain::user::session::Token);

    /// Result of a `Session` creation.
    #[derive(Clone, Debug, From, GraphQLObject)]
    #[graphql(context = Context, name = "CreateSessionResult")]
    pub struct CreateResult {
        /// Access token of the created `Session`.
        pub token: Token,

        /// `User` associated with the created `Session`.
        pub user: api::User,

        /// `DateTime` when the created `Session` expires.
        pub expires_at: DateTime,
    }

    impl From<command::create_user_session::Output> for CreateResult {
        fn from(output: command::create_user_session::Output) -> Self {
            let command::create_user_session::Output {
                token,
                user,
                expires_at,
            } = output;
            Self {
                token: token.into(),
                user: user.into(),
                expires_at: expires_at.coerce(),
            }
        }
    }
}

pub mod list {
    //! Definitions related to [`User`] list.

    use derive_more::{AsRef, From, Into};
    use juniper::{graphql_object, GraphQLScalar};
    use service::{query, read, Query as _};

    use crate::{api::scalar, AsError, Context, Error};

    use super::{Id, User};

    /// Cursor for the `User` list.
    #[derive(AsRef, Clone, Copy, Debug, From, GraphQLScalar, Into)]
    #[from(Id, read::user::list::Cursor)]
    #[graphql(
        name = "UserListCursor",
        with = scalar::Via::<read::user::list::Cursor>,
    )]
    pub struct Cursor(pub read::user::list::Cursor);

    /// Edge in the [`User`] list.
    #[derive(Clone, Copy, Debug, From, Into)]
    pub struct Edge(read::user::list::Edge);

    /// Edge in the `User` list.
    #[graphql_object(name = "UserListEdge", context = Context)]
    impl Edge {
        /// Cursor of this `UserListEdge`.
        #[must_use]
        pub fn cursor(&self) -> Cursor {
            self.0.cursor.into()
        }

        /// Node of this `UserListEdge`.
        #[must_use]
        pub fn node(&self) -> User {
            #[expect(
                unsafe_code,
                reason = "`Edge` loaded from repository guarantees `User` \
                          existence"
            )]
            unsafe {
                User::new_unchecked(self.0.node)
            }
        }
    }

    /// Connection of the [`User`] list.
    #[derive(Clone, Debug, From, Into)]
    pub struct Connection(read::user::list::Connection);

    /// Connection of the `User` list.
    #[graphql_object(name = "UserListConnection", context = Context)]
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
        /// Underlying [`read::user::list::PageInfo`].
        info: read::user::list::PageInfo,

        /// Start cursor of the page.
        start_cursor: Option<Cursor>,

        /// End cursor of the page.
        end_cursor: Option<Cursor>,
    }

    /// Information about a `UserListConnection` page.
    #[graphql_object(name = "UserListPageInfo", context = Context)]
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

        /// Total `User`s count.
        pub async fn total_count(&self, ctx: &Context) -> Result<i32, Error> {
            ctx.service()
                .execute(query::users::TotalCount::by(()))
                .await
                .map_err(AsError::into_error)
                .map_err(ctx.error())
                .map(Into::into)
        }
    }
}
