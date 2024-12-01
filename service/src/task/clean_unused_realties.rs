//! [`CleanUnusedRealties`] [`Task`].

use std::{convert::Infallible, error::Error, time};

use common::operations::{By, Delete, Perform, Start};
use tokio::time::interval;
use tracerr::Traced;
use tracing as log;

use crate::{
    domain::{realty, Realty},
    infra::{database, Database},
    Service,
};

use super::Task;

/// Configuration for [`CleanUnusedRealties`] [`Task`].
#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Interval between [`Realty`] entities cleaning.
    pub interval: time::Duration,

    /// Timeout after which a [`Realty`] is considered unused.
    pub timeout: time::Duration,
}

/// [`Task`] for cleaning unused [`Realty`] entities.
#[derive(Clone, Copy, Debug)]
pub struct CleanUnusedRealties<S> {
    /// [`Config`] of this [`Task`].
    config: Config,

    /// [`Service`] instance.
    service: S,
}

impl<Db> Task<Start<By<CleanUnusedRealties<Self>, Config>>> for Service<Db>
where
    CleanUnusedRealties<Service<Db>>:
        Task<Perform<()>, Ok = (), Err: Error> + Send + Sync + 'static,
    Self: Clone,
{
    type Ok = ();
    type Err = Infallible;

    async fn execute(
        &self,
        Start(by): Start<By<CleanUnusedRealties<Self>, Config>>,
    ) -> Result<Self::Ok, Self::Err> {
        let config = by.into_inner();
        let task = CleanUnusedRealties {
            config,
            service: self.clone(),
        };

        let mut interval = interval(task.config.interval);
        loop {
            let _ = interval.tick().await;
            _ = task.execute(Perform(())).await.map_err(|e| {
                log::error!("`task::CleanUnusedRealties` failed: {e}");
            });
        }
    }
}

impl<Db> Task<Perform<()>> for CleanUnusedRealties<Service<Db>>
where
    Db: Database<
        Delete<By<Realty, realty::CreationDateTime>>,
        Ok = (),
        Err = Traced<database::Error>,
    >,
{
    type Ok = ();
    type Err = ExecutionError;

    async fn execute(&self, _: Perform<()>) -> Result<Self::Ok, Self::Err> {
        let deadline = realty::CreationDateTime::now() - self.config.timeout;
        self.service
            .database()
            .execute(Delete(By::new(deadline)))
            .await
            .map_err(tracerr::map_from_and_wrap!())
    }
}

/// Error of [`CleanUnusedRealties`] execution.
pub type ExecutionError = Traced<database::Error>;
