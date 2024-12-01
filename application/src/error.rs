//! [`Error`]-related definitions.

use std::{fmt, num::TryFromIntError};

use axum_extra::typed_header::TypedHeaderRejection;
use derive_more::Error as StdError;
use itertools::Itertools as _;
use juniper::IntoFieldError;
use service::infra::database;
use tracerr::{Trace, Traced};

/// Defines a new error type.
#[expect(clippy::module_name_repetitions, reason = "more readable")]
#[macro_export]
macro_rules! define_error {
    (
        enum $name:ident {
            $(
                #[code = $code:literal]
                #[status = $status_code:ident]
                #[message = $message:literal]
                $variant:ident
            ),* $(,)?
        }
    ) => {
        /// Error type.
        #[derive(
            Clone,
            Copy,
            Debug,
            ::derive_more::Display,
            ::derive_more::Error
        )]
        #[repr(u16)]
        pub enum $name {
            $(
                #[display($message)]
                #[doc = $message]
                $variant,
            )*
        }

        impl From<$name> for $crate::Error {
            fn from(err: $name) -> Self {
                match err {
                    $(
                        $name::$variant => Self {
                            code: $code,
                            status_code: ::http::StatusCode::$status_code,
                            message: $message.to_string(),
                            backtrace: None,
                        },
                    )*
                }
            }
        }
    };
}

/// GraphQL API [`Error`].
#[derive(Clone, Debug, StdError)]
pub struct Error {
    /// [`Error`] code.
    pub code: Code,

    /// [`http::StatusCode`] of this [`Error`].
    pub status_code: http::StatusCode,

    /// Backtrace of this [`Error`].
    #[error(not(backtrace))]
    pub backtrace: Option<Trace>,

    /// [`Error`] message.
    pub message: String,
}

impl Error {
    /// Create a new [`Error`] representing an internal server error.
    #[must_use]
    pub fn internal(msg: &impl ToString) -> Self {
        Self {
            code: "INTERNAL_SERVER_ERROR",
            status_code: http::StatusCode::INTERNAL_SERVER_ERROR,
            message: msg.to_string(),
            backtrace: None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            code,
            status_code: _,
            backtrace,
            message,
        } = self;

        write!(
            f,
            "[{code}]: {message}{}",
            backtrace
                .iter()
                .format_with("\n", |trace, f| f(&format_args!("{trace}"))),
        )
    }
}

impl<S> IntoFieldError<S> for Error
where
    S: From<String>,
{
    fn into_field_error(self) -> juniper::FieldError<S> {
        let mut ext = juniper::Object::with_capacity(1);
        drop(
            ext.add_field("code", juniper::Value::scalar(self.code.to_owned())),
        );
        drop(
            ext.add_field(
                "backtrace",
                juniper::Value::list(
                    self.backtrace
                        .iter()
                        .flat_map(|trace| trace.iter())
                        .map(|frame| juniper::Value::scalar(frame.to_string()))
                        .collect(),
                ),
            ),
        );
        juniper::FieldError::new(self.message, juniper::Value::object(ext))
    }
}

/// [`Error`] code.
pub type Code = &'static str;

/// Helper trait for converting types into [`Error`]s.
pub trait AsError {
    /// Tries to convert the type into an [`Error`].
    ///
    /// [`None`] is returned if the type cannot be converted into an [`Error`].
    fn try_as_error(&self) -> Option<Error>;

    /// Converts the type into an [`Error`].
    fn as_error(&self) -> Error
    where
        Self: fmt::Display,
    {
        self.try_as_error()
            .unwrap_or_else(|| Error::internal(&self))
    }

    /// Converts the type into an [`Error`] by consuming it.
    fn into_error(self) -> Error
    where
        Self: fmt::Display + Sized,
    {
        self.as_error()
    }
}

impl<E: AsError> AsError for Traced<E> {
    fn try_as_error(&self) -> Option<Error> {
        let mut error = self.as_ref().try_as_error()?;
        error.backtrace = Some(self.trace().clone());
        Some(error)
    }
}

impl AsError for TypedHeaderRejection {
    fn try_as_error(&self) -> Option<Error> {
        Some(Error {
            code: "BAD_REQUEST",
            status_code: http::StatusCode::BAD_REQUEST,
            message: self.to_string(),
            backtrace: None,
        })
    }
}

impl AsError for database::Error {
    fn try_as_error(&self) -> Option<Error> {
        None
    }
}

impl AsError for TryFromIntError {
    fn try_as_error(&self) -> Option<Error> {
        None
    }
}
