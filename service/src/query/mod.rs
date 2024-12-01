//! [`Query`] definition.

pub mod contract;
pub mod contracts;
pub mod placements;
pub mod realties;
pub mod realty;
pub mod report;
pub mod user;
pub mod users;

use common::operations::{By, Select};
use tracerr::Traced;

use crate::{
    infra::{database, Database},
    Service,
};

/// [`Query`] of the [`Service`].
pub use common::Handler as Query;

/// [`Query`] [`Select`]ing a `T`ype from a [`Database`].
#[derive(Clone, Copy, Debug)]
#[expect(clippy::module_name_repetitions, reason = "more readable")]
pub struct DatabaseQuery<T>(T);

impl<W, B> DatabaseQuery<By<W, B>> {
    /// Creates a new [`DatabaseQuery`] selecting a `W` by the provided `B`.
    #[must_use]
    pub fn by(by: B) -> Self {
        Self(By::new(by))
    }
}

impl<Db, W, B> Query<DatabaseQuery<By<W, B>>> for Service<Db>
where
    Db: Database<Select<By<W, B>>, Ok = W, Err = Traced<database::Error>>,
{
    type Ok = W;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        DatabaseQuery(by): DatabaseQuery<By<W, B>>,
    ) -> Result<Self::Ok, Self::Err> {
        self.database()
            .execute(Select(by))
            .await
            .map_err(tracerr::wrap!())
    }
}
