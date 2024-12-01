//! Postgres [`Database`] implementation.

pub mod client;
pub mod connection;
mod fuzz_pattern;
mod impls;

use deadpool_postgres::Runtime;
use derive_more::{Deref, Display, Error as StdError, From};
use tokio_postgres::{error::SqlState, NoTls};
use tracerr::Traced;

use crate::infra::database;
#[cfg(doc)]
use crate::infra::Database;

pub use refinery::embed_migrations;

pub use self::{
    client::{NonTx, Tx},
    connection::Connection,
    fuzz_pattern::FuzzPattern,
};

pub use deadpool_postgres::Config;

/// Postgres [`Database`] client.
#[derive(Clone, Copy, Debug, Deref)]
pub struct Postgres<T = NonTx>(T);

impl Postgres {
    /// Creates a new [`Postgres`] client with the provided [`Config`].
    ///
    /// # Errors
    ///
    /// If failed to create a new [`Postgres`] client.
    pub fn new(conf: &Config) -> Result<Self, Traced<database::Error>> {
        let pool = conf
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(tracerr::from_and_wrap!(=> Error))
            .map_err(tracerr::map_from)?;
        Ok(Self(NonTx::from_pool(pool)))
    }
}

/// Postgres database [`Error`].
#[derive(Debug, Display, StdError, From)]
pub enum Error {
    /// [`Connection`] error.
    #[display("`Connection` error: {_0}")]
    Connection(connection::Error),

    /// Error of creating a new [`connection::Pool`] client.
    #[display("Failed to create a new `connection::Pool`: {_0}")]
    PoolCreationError(connection::PoolCreationError),

    /// [`connection::Pool`] error.
    #[display("`connection::Pool` error: {_0}")]
    PoolError(connection::PoolError),
}

impl Error {
    /// Checks if the error is a unique violation of the specified constraint.
    #[must_use]
    pub fn is_unique_violation(&self, constraint: Option<&str>) -> bool {
        match self {
            Self::Connection(e) => {
                e.code() == Some(&SqlState::UNIQUE_VIOLATION)
                    && constraint.map_or(true, |c| {
                        e.as_db_error().and_then(|e| e.constraint()) == Some(c)
                    })
            }
            Self::PoolError(..) | Self::PoolCreationError(..) => false,
        }
    }
}
