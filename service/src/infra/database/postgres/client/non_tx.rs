//! [`NonTx`] client definitions.

use std::sync::Arc;

use tokio::sync::{RwLock, RwLockReadGuard};
use tokio_postgres::{types::ToSql, Row, ToStatement};
use tracerr::Traced;

use crate::infra::database::{
    self,
    postgres::{self, connection, Connection},
};

/// Non-transactional Postgres database client.
#[derive(Clone, Debug)]
pub struct NonTx {
    /// [`connection::Pool`] to initialize the client.
    pub(crate) pool: connection::Pool,

    /// Client to be used for non-transactional operations, if any.
    connection: Arc<RwLock<Option<connection::NonTx>>>,
}

impl NonTx {
    /// Creates a new [`NonTx`] client from the provided [`connection::Pool`].
    #[must_use]
    pub(crate) fn from_pool(pool: connection::Pool) -> Self {
        Self {
            pool,
            connection: Arc::new(RwLock::new(None)),
        }
    }

    /// Returns the underlying [`Connection`] of this [`NonTx`] client.
    pub(crate) async fn connection(
        &self,
    ) -> Result<RwLockReadGuard<'_, connection::NonTx>, Traced<database::Error>>
    {
        let connection = self.connection.read().await;
        let guard = if connection.is_none() {
            drop(connection);

            let mut connection = self.connection.write().await;
            if connection.is_none() {
                *connection = Some(
                    self.pool
                        .get()
                        .await
                        .map_err(tracerr::from_and_wrap!(=> postgres::Error))
                        .map_err(tracerr::map_from)?,
                );
            }

            connection.downgrade()
        } else {
            connection
        };

        Ok(RwLockReadGuard::map(guard, |conn| {
            conn.as_ref()
                .expect("connection cannot be dropped while guard is alive")
        }))
    }

    /// Takes the underlying [`Connection`] from this [`NonTx`] client.
    ///
    /// Next time this [`NonTx`] client is used, it will initialize a new
    /// [`Connection`].
    #[must_use]
    pub(crate) async fn take_connection(&self) -> Option<connection::NonTx> {
        self.connection.write().await.take()
    }
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
        self.connection()
            .await
            .map_err(tracerr::wrap!())?
            .query(stmt, params)
            .await
            .map_err(tracerr::wrap!())
    }

    async fn query_opt<T>(
        &self,
        stmt: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Option<Row>, Traced<database::Error>>
    where
        T: ToStatement + ?Sized,
    {
        self.connection()
            .await
            .map_err(tracerr::wrap!())?
            .query_opt(stmt, params)
            .await
            .map_err(tracerr::wrap!())
    }

    async fn exec<T>(
        &self,
        stmt: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<u64, Traced<database::Error>>
    where
        T: ToStatement + ?Sized,
    {
        self.connection()
            .await
            .map_err(tracerr::wrap!())?
            .exec(stmt, params)
            .await
            .map_err(tracerr::wrap!())
    }

    async fn batch_exec(
        &self,
        query: &str,
    ) -> Result<(), Traced<database::Error>> {
        self.connection()
            .await
            .map_err(tracerr::wrap!())?
            .batch_exec(query)
            .await
            .map_err(tracerr::wrap!())
    }
}
