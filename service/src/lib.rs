//! Service contains the business logic of the application.
//!
//! List of available Cargo features:
#![doc = document_features::document_features!()]
#![deny(
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::all,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code
)]
#![forbid(non_ascii_idents)]
#![warn(
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    clippy::pedantic,
    clippy::wildcard_enum_match_arm,
    deprecated_in_future,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    unused_crate_dependencies,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]

pub mod command;
pub mod domain;
pub mod infra;
pub mod query;
pub mod read;
pub mod task;

use common::operations::{By, Start};
use derive_more::{Debug, Display, Error};

#[cfg(doc)]
use infra::Database;

pub use self::{command::Command, query::Query, task::Task};

/// [`Service`] configuration.
#[derive(Clone, Debug)]
pub struct Config {
    /// [JWT] encoding key.
    ///
    /// [JWT]: https://datatracker.ietf.org/doc/html/rfc7519
    #[debug(skip)]
    pub jwt_encoding_key: jsonwebtoken::EncodingKey,

    /// [JWT] decoding key.
    ///
    /// [JWT]: https://datatracker.ietf.org/doc/html/rfc7519
    #[debug(skip)]
    pub jwt_decoding_key: jsonwebtoken::DecodingKey,

    /// [`task::CleanUnusedRealties`] configuration.
    pub clean_unused_realties: task::clean_unused_realties::Config,
}

/// Domain service.
#[derive(Clone, Debug)]
pub struct Service<Db> {
    /// Configuration of this [`Service`].
    config: Config,

    /// [`Database`] of this [`Service`].
    database: Db,
}

impl<Db> Service<Db> {
    /// Creates a new [`Service`] with the provided parameters.
    pub fn new(config: Config, database: Db) -> (Self, task::Background)
    where
        Self: Task<
                Start<
                    By<
                        task::CleanUnusedRealties<Self>,
                        task::clean_unused_realties::Config,
                    >,
                >,
                Ok = (),
                Err: Error,
            > + Clone
            + 'static,
    {
        let this = Service { config, database };

        let mut bg = task::Background::default();
        let svc = this.clone();
        bg.spawn(async move {
            svc.execute(Start(By::new(svc.config().clean_unused_realties)))
                .await
        });

        (this, bg)
    }

    /// Returns [`Config`] of this [`Service`].
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Returns [`Database`] of this [`Service`].
    #[must_use]
    pub fn database(&self) -> &Db {
        &self.database
    }
}

/// Shortcut for the error of starting a [`Task`].
type TaskStartError<Svc, T, Args> = <Svc as Task<Start<By<T, Args>>>>::Err;

/// Error of starting a [`Service`].
#[derive(Debug, Display, Error)]
pub enum StartupError<Svc>
where
    Svc: Task<
        Start<
            By<
                task::CleanUnusedRealties<Svc>,
                task::clean_unused_realties::Config,
            >,
        >,
    >,
{
    /// [`task::CleanUnusedRealties`] failed to start.
    CleanUnusedRealtiesTask(
        TaskStartError<
            Svc,
            task::CleanUnusedRealties<Svc>,
            task::clean_unused_realties::Config,
        >,
    ),
}
