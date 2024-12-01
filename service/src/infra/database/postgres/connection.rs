//! [`Connection`] definitions.

use std::{fmt, future::Future};

use futures::{FutureExt as _, TryFutureExt as _};
use ouroboros::self_referencing;
use tokio_postgres::{types::ToSql, Row, ToStatement};
use tracerr::Traced;

use crate::infra::database::{self, postgres};

pub use deadpool_postgres::{
    Client as NonTx, CreatePoolError as PoolCreationError, Pool, PoolError,
};
pub use tokio_postgres::Error;

/// Transactional Postgres database [`Connection`].
#[self_referencing]
pub struct Tx {
    /// [`NonTx`] [`Connection`] the transaction was started from.
    non_tx: NonTx,

    /// Transaction started in the [`Connection`].
    #[borrows(mut non_tx)]
    #[not_covariant]
    tx: Option<deadpool_postgres::Transaction<'this>>,
}

impl fmt::Debug for Tx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tx")
            .field("tx", self.tx())
            .finish_non_exhaustive()
    }
}

impl Tx {
    /// Returns the underlying [`Transaction`] of this [`Tx`] connection.
    ///
    /// [`Transaction`]: deadpool_postgres::Transaction
    fn tx(&self) -> &deadpool_postgres::Transaction<'_> {
        self.with_tx(|tx| tx.as_ref().expect("already committed"))
    }

    /// Creates a new [`Tx`] from the provided [`NonTx`] [`Connection`].
    ///
    /// # Errors
    ///
    /// If failed to create a new [`Tx`] from the provided [`NonTx`].
    pub async fn from_non_tx(
        client: NonTx,
    ) -> Result<Tx, Traced<database::Error>> {
        Tx::try_new_async_send(client, |c| c.transaction().map_ok(Some).boxed())
            .await
            .map_err(tracerr::from_and_wrap!(=> postgres::Error))
            .map_err(tracerr::map_from)
    }

    /// Commits this [`Tx`].
    ///
    /// # Errors
    ///
    /// If failed to commit this [`Tx`].
    #[expect(clippy::missing_panics_doc, reason = "infallible")]
    pub async fn commit(mut self) -> Result<(), Traced<database::Error>> {
        #[expect(
            clippy::redundant_closure_for_method_calls,
            reason = "different variance, see \
                      https://doc.rust-lang.org/nomicon/subtyping.html#variance"
        )]
        self.with_tx_mut(|tx| tx.take())
            .expect("already committed")
            .commit()
            .await
            .map_err(tracerr::from_and_wrap!(=> postgres::Error))
            .map_err(tracerr::map_from)
    }
}

/// Generic database connection.
pub trait Connection {
    /// Queries the provided statement with the given parameters and returns the
    /// resulting rows.
    ///
    /// # Errors
    ///
    /// If failed to query the statement.
    fn query<T>(
        &self,
        stmt: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> impl Future<Output = Result<Vec<Row>, Traced<database::Error>>>
    where
        T: ToStatement + ?Sized;

    /// Queries the provided statement with the given parameters and returns the
    /// optional resulting row.
    ///
    /// # Errors
    ///
    /// If failed to query the statement.
    fn query_opt<T>(
        &self,
        stmt: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> impl Future<Output = Result<Option<Row>, Traced<database::Error>>>
    where
        T: ToStatement + ?Sized;

    /// Executes the provided statement with the given parameters and returns
    /// the number of affected rows.
    ///
    /// # Errors
    ///
    /// If failed to execute the statement.
    fn exec<T>(
        &self,
        stmt: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> impl Future<Output = Result<u64, Traced<database::Error>>>
    where
        T: ToStatement + ?Sized;

    /// Executes the provided batch query.
    ///
    /// # Errors
    ///
    /// If failed to execute the batch query.
    fn batch_exec(
        &self,
        stmt: &str,
    ) -> impl Future<Output = Result<(), Traced<database::Error>>>;
}

impl Connection for NonTx {
    async fn query<T>(
        &self,
        stmt: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, Traced<database::Error>>
    where
        T: ToStatement + ?Sized,
    {
        (**self)
            .query(stmt, params)
            .await
            .map_err(tracerr::from_and_wrap!(=> postgres::Error))
            .map_err(tracerr::map_from)
    }

    async fn query_opt<T>(
        &self,
        stmt: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Option<Row>, Traced<database::Error>>
    where
        T: ToStatement + ?Sized,
    {
        (**self)
            .query_opt(stmt, params)
            .await
            .map_err(tracerr::from_and_wrap!(=> postgres::Error))
            .map_err(tracerr::map_from)
    }

    async fn exec<T>(
        &self,
        stmt: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<u64, Traced<database::Error>>
    where
        T: ToStatement + ?Sized,
    {
        (**self)
            .execute(stmt, params)
            .await
            .map_err(tracerr::from_and_wrap!(=> postgres::Error))
            .map_err(tracerr::map_from)
    }

    async fn batch_exec(
        &self,
        query: &str,
    ) -> Result<(), Traced<database::Error>> {
        (**self)
            .batch_execute(query)
            .await
            .map_err(tracerr::from_and_wrap!(=> postgres::Error))
            .map_err(tracerr::map_from)
    }
}

impl Connection for Tx {
    async fn query<T>(
        &self,
        stmt: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, Traced<database::Error>>
    where
        T: ToStatement + ?Sized,
    {
        self.tx()
            .query(stmt, params)
            .await
            .map_err(tracerr::from_and_wrap!(=> postgres::Error))
            .map_err(tracerr::map_from)
    }

    async fn query_opt<T>(
        &self,
        stmt: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Option<Row>, Traced<database::Error>>
    where
        T: ToStatement + ?Sized,
    {
        self.tx()
            .query_opt(stmt, params)
            .await
            .map_err(tracerr::from_and_wrap!(=> postgres::Error))
            .map_err(tracerr::map_from)
    }

    async fn exec<T>(
        &self,
        stmt: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<u64, Traced<database::Error>>
    where
        T: ToStatement + ?Sized,
    {
        self.tx()
            .execute(stmt, params)
            .await
            .map_err(tracerr::from_and_wrap!(=> postgres::Error))
            .map_err(tracerr::map_from)
    }

    async fn batch_exec(
        &self,
        query: &str,
    ) -> Result<(), Traced<database::Error>> {
        self.tx()
            .batch_execute(query)
            .await
            .map_err(tracerr::from_and_wrap!(=> postgres::Error))
            .map_err(tracerr::map_from)
    }
}
