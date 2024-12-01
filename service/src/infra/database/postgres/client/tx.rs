//! [`Tx`] client definitions.

use std::sync::Arc;

use tokio::sync::{RwLock, RwLockReadGuard};
use tokio_postgres::{types::ToSql, Row, ToStatement};
use tracerr::Traced;

use crate::infra::database::{
    self,
    postgres::{self, connection, Connection},
};

use super::NonTx;

/// Transactional Postgres database client.
#[derive(Clone, Debug)]
pub struct Tx {
    /// [`connection::Pool`] to retrieve the [`Connection`] from.
    pool: connection::Pool,

    /// Inner representation of this client.
    inner: Arc<Inner>,
}

/// Inner representation of the [`Tx`] client.
#[derive(Debug)]
pub struct Inner {
    /// [`NonTx`] client to initialize the [`connection::Tx`] from, if any.
    non_tx: RwLock<Option<NonTx>>,

    /// Lazily initialized [`connection::Tx`].
    tx: Arc<RwLock<Option<connection::Tx>>>,
}

impl Tx {
    /// Creates a new [`Tx`] client from the provided [`NonTx`] client.
    #[must_use]
    pub fn from_non_tx(client: NonTx) -> Self {
        Self {
            pool: client.pool.clone(),
            inner: Arc::new(Inner {
                non_tx: RwLock::new(Some(client)),
                tx: Arc::new(RwLock::new(None)),
            }),
        }
    }

    /// Returns underlying [`Connection`] of this [`Tx`] client.
    async fn connection(
        &self,
    ) -> Result<RwLockReadGuard<'_, connection::Tx>, Traced<database::Error>>
    {
        let connection = self.inner.tx.read().await;
        let guard = if connection.is_none() {
            drop(connection);

            let mut connection = self.inner.tx.write().await;
            if connection.is_none() {
                let mut existing = None;
                if self.inner.non_tx.read().await.is_some() {
                    if let Some(cl) = self.inner.non_tx.write().await.take() {
                        if let Some(conn) = cl.take_connection().await {
                            existing = Some(conn);
                        }
                    }
                }

                let conn = if let Some(c) = existing {
                    c
                } else {
                    self.pool
                        .get()
                        .await
                        .map_err(tracerr::from_and_wrap!(=> postgres::Error))
                        .map_err(tracerr::map_from)?
                };

                *connection = Some(
                    connection::Tx::from_non_tx(conn)
                        .await
                        .map_err(tracerr::wrap!())?,
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

    /// Takes the underlying [`Connection`] from this [`Tx`] client.
    ///
    /// Next time this [`Tx`] client is used, it will initialize a new
    /// [`Connection`].
    async fn take_connection(&self) -> Option<connection::Tx> {
        self.inner.tx.write().await.take()
    }

    /// Commits this [`Tx`] client.
    ///
    /// # Errors
    ///
    /// If failed to commit transaction of this [`Tx`] client.
    pub async fn commit(&self) -> Result<(), Traced<database::Error>> {
        if let Some(tx) = self.take_connection().await {
            tx.commit().await.map_err(tracerr::wrap!())
        } else {
            // No transaction to commit, so nothing to do.
            Ok(())
        }
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
