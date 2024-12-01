/// [`Context`]-related definitions.
use std::{
    future,
    sync::atomic::{self, AtomicU16},
};

use axum::{async_trait, extract::FromRequestParts, RequestPartsExt as _};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use common::DateTime;
use juniper::{
    http::{GraphQLBatchResponse, GraphQLResponse},
    IntoFieldError as _,
};
use service::{
    command::{self, Command as _},
    domain::user::session,
};
use tokio::sync::OnceCell;

#[cfg(doc)]
use crate::api::User;
use crate::{api, define_error, AsError, Error, JuniperResponse, Service};

/// Application context.
#[derive(Debug)]
pub struct Context {
    /// [`Service`] instance.
    service: Service,

    /// Error status code.
    error_status_code: AtomicU16,

    /// Parts of the HTTP request.
    parts: http::request::Parts,

    /// Current [`Session`].
    current_session: OnceCell<Session>,

    /// Last authentication [`Error`].
    auth_error: OnceCell<Error>,
}

impl Context {
    /// Returns [`Service`] instance of this [`Context`].
    #[must_use]
    pub fn service(&self) -> &Service {
        &self.service
    }

    /// Returns the error status code of this [`Context`].
    #[expect(clippy::missing_panics_doc, reason = "infallible")]
    #[must_use]
    pub fn error_status_code(&self) -> http::StatusCode {
        http::StatusCode::from_u16(
            self.error_status_code.load(atomic::Ordering::Relaxed),
        )
        .expect("invalid status code")
    }

    /// Sets the error status code for this [`Context`].
    ///
    /// Provided [`http::StatusCode`] will be applied to the response.
    pub fn set_error_status_code(&self, status_code: http::StatusCode) {
        self.error_status_code
            .store(status_code.as_u16(), atomic::Ordering::Relaxed);
    }

    /// Helper method calling [`Context::set_error_status_code()`] inside
    /// [`Result::map_err()`] closure.
    pub fn error(&self) -> impl FnOnce(Error) -> Error + '_ {
        move |err| {
            self.set_error_status_code(err.status_code);
            err
        }
    }

    /// Sets the current [`Session`] for this [`Context`].
    pub async fn set_current_session(&self, session: Session) {
        _ = self
            .current_session
            .get_or_init(|| future::ready(session))
            .await;
    }

    /// Tries to get the current [`Session`] for this [`Context`].
    ///
    /// # Errors
    ///
    /// Errors if the provided authentication token is invalid.
    pub async fn try_current_session(&self) -> Result<Option<Session>, Error> {
        self.current_session().await.map(Some).or_else(|e| {
            if e.code == Error::from(AuthError::AuthroizationRequired).code {
                Ok(None)
            } else {
                Err(e)
            }
        })
    }

    /// Returns the current [`Session`] for this [`Context`].
    ///
    /// # Errors
    ///
    /// Errors if:
    /// - the current HTTP request is not authorized;
    /// - the provided authentication token is invalid.
    pub async fn current_session(&self) -> Result<Session, Error> {
        self.current_session
            .get_or_try_init(|| async {
                match self
                    .auth_error
                    .get_or_try_init(|| async {
                        match self.do_authentication().await {
                            Ok(u) => Err(u),
                            Err(e) => Ok(e),
                        }
                    })
                    .await
                {
                    Ok(e) => Err(e),
                    Err(u) => Ok(u),
                }
            })
            .await
            .cloned()
            .map_err(Clone::clone)
    }

    /// Applies the [`juniper::Variables`] provided by the client on GraphQL
    /// subscription initialization.
    ///
    /// # Errors
    ///
    /// Errors if the provided variables are invalid.
    pub(crate) fn apply_subscription_variables(
        &mut self,
        vars: &juniper::Variables,
    ) -> Result<(), Error> {
        if let Some(token) = vars.get("authToken") {
            let token = token
                .as_string_value()
                .ok_or_else(|| Error::from(AuthError::InvalidVariables))?;
            let token = format!("Bearer {token}")
                .parse()
                .map_err(|_| Error::from(AuthError::InvalidVariables))?;
            drop(
                self.parts
                    .headers
                    .insert(http::header::AUTHORIZATION, token),
            );
        }

        Ok(())
    }

    /// Performs the [`Session`] authentication.
    ///
    /// # Errors
    ///
    /// Errors if the provided authentication token is invalid.
    async fn do_authentication(&self) -> Result<Session, Error> {
        let res = self
            .parts
            .clone()
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await;
        match res {
            Ok(TypedHeader(Authorization(bearer))) => {
                #[expect(unsafe_code, reason = "specified in correct header")]
                let token = unsafe {
                    session::Token::new_unchecked(bearer.token().to_owned())
                };
                self.service
                    .execute(command::AuthorizeUserSession {
                        token: token.clone(),
                    })
                    .await
                    .map(|s| Session {
                        user_id: s.user_id.into(),
                        token,
                        expires_at: s.expires_at.coerce(),
                    })
                    .map_err(AsError::into_error)
            }
            Err(e) => {
                if e.is_missing() {
                    Err(AuthError::AuthroizationRequired.into())
                } else {
                    Err(e.into_error())
                }
            }
        }
        .map_err(self.error())
    }
}

impl juniper::Context for Context {}

#[async_trait]
impl<S> FromRequestParts<S> for Context
where
    S: Send + Sync,
{
    type Rejection = JuniperResponse;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        let service =
            parts.extensions.get::<Service>().cloned().ok_or_else(|| {
                JuniperResponse {
                    status_code: http::StatusCode::INTERNAL_SERVER_ERROR,
                    response: GraphQLBatchResponse::Single(
                        GraphQLResponse::error(
                            Error::internal(&"missing `Service` extension")
                                .into_field_error(),
                        ),
                    ),
                }
            })?;

        Ok(Self {
            service,
            error_status_code: AtomicU16::new(
                http::StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            ),
            parts: parts.clone(),
            current_session: OnceCell::new(),
            auth_error: OnceCell::new(),
        })
    }
}

/// User session.
#[derive(Clone, Debug)]
pub struct Session {
    /// ID of the [`User`] associated with this [`Session`].
    pub user_id: api::user::Id,

    /// Authentication token.
    pub token: session::Token,

    /// [`DateTime`] when this [`Session`] expires.
    pub expires_at: DateTime,
}

impl AsError for command::authorize_user_session::ExecutionError {
    fn try_as_error(&self) -> Option<Error> {
        match self {
            Self::Db(e) => e.try_as_error(),
            Self::JsonWebTokenDecodeError(_) => {
                Some(AuthError::AuthroizationRequired.into())
            }
            Self::UserNotExists(_) => None,
        }
    }
}

define_error! {
    enum AuthError {
        #[code = "AUTHORIZATION_REQUIRED"]
        #[status = UNAUTHORIZED]
        #[message = "Authorization required"]
        AuthroizationRequired,

        #[code = "INVALID_VARIABLES"]
        #[status = BAD_REQUEST]
        #[message = "Invalid subscription authorization variables"]
        InvalidVariables,
    }
}
