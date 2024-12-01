//! [`Database`] implementations.

#![allow(
    clippy::items_after_statements,
    reason = "`const SQL` after statements"
)]
#![allow(clippy::too_many_lines, reason = "SQL-related code a bit verbose")]

mod contract;
mod placement;
mod realty;
mod user;

use async_trait::async_trait;
use common::operations::{Commit, Transact};
use refinery_core::{
    traits::r#async::{AsyncQuery, AsyncTransaction},
    AsyncMigrate, Migration,
};
use tracerr::Traced;

use crate::infra::{database, postgres, Database};

use super::{NonTx, Postgres, Tx};

impl Database<Transact> for Postgres<NonTx> {
    type Ok = Postgres<Tx>;
    type Err = Traced<database::Error>;

    async fn execute(&self, _: Transact) -> Result<Self::Ok, Self::Err> {
        Ok(Postgres(Tx::from_non_tx(self.0.clone())))
    }
}

impl Database<Transact> for Postgres<Tx> {
    type Ok = Self;
    type Err = Traced<database::Error>;

    async fn execute(&self, _: Transact) -> Result<Self::Ok, Self::Err> {
        Ok(self.clone())
    }
}

impl Database<Commit> for Postgres<Tx> {
    type Ok = ();
    type Err = Traced<database::Error>;

    async fn execute(&self, _: Commit) -> Result<Self::Ok, Self::Err> {
        self.commit().await.map_err(tracerr::wrap!())
    }
}

#[async_trait]
impl AsyncTransaction for Postgres {
    type Error = Traced<database::Error>;

    async fn execute(
        &mut self,
        queries: &[&str],
    ) -> Result<usize, Self::Error> {
        let mut conn = self
            .0
            .pool
            .get()
            .await
            .map_err(tracerr::from_and_wrap!(=> postgres::Error))
            .map_err(tracerr::map_from)?;
        AsyncTransaction::execute(&mut **conn, queries)
            .await
            .map_err(tracerr::from_and_wrap!(=> postgres::Error))
            .map_err(tracerr::map_from)
    }
}

#[async_trait]
impl AsyncQuery<Vec<Migration>> for Postgres {
    async fn query(
        &mut self,
        query: &str,
    ) -> Result<Vec<Migration>, <Self as AsyncTransaction>::Error> {
        let mut conn = self
            .0
            .pool
            .get()
            .await
            .map_err(tracerr::from_and_wrap!(=> postgres::Error))
            .map_err(tracerr::map_from)?;
        AsyncQuery::query(&mut **conn, query)
            .await
            .map_err(tracerr::from_and_wrap!(=> postgres::Error))
            .map_err(tracerr::map_from)
    }
}

impl AsyncMigrate for Postgres {}
