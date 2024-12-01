//! Infrastructure layer.

pub mod database;

pub use self::database::Database;
#[cfg(feature = "postgres")]
pub use self::database::{postgres, Postgres};
