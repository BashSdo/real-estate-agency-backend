//! [`Database`]-related implementations.

#[cfg(feature = "postgres")]
pub mod postgres;

use derive_more::{Display, Error as StdError, From};

#[cfg(feature = "postgres")]
pub use self::postgres::Postgres;

/// Database operation.
pub use common::Handler as Database;

/// [`Database`] error.
#[derive(Debug, Display, From, StdError)]
pub enum Error {
    #[cfg(feature = "postgres")]
    /// [`Postgres`] error.
    Postgres(postgres::Error),
}
