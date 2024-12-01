//! [`Command`] for updating an [`user::Phone`].

use common::operations::{
    By, Commit, Lock, Select, Transact, Transacted, Update,
};
use derive_more::{Display, Error, From};
use tracerr::Traced;

#[cfg(doc)]
use crate::domain::user::Phone;
use crate::{
    domain::{user, User},
    infra::{database, Database},
    Service,
};

use super::Command;

/// [`Command`] for updating an [`user::Phone`].
#[derive(Clone, Debug, From)]
pub struct UpdateUserPhone {
    /// ID of the [`User`] which [`Phone`] should be updated.
    pub user_id: user::Id,

    /// New [`Phone`] number of the [`User`].
    ///
    /// [`None`] indicating [`Phone`] deletion.
    pub number: Option<user::Phone>,
}

impl<Db> Command<UpdateUserPhone> for Service<Db>
where
    Db: Database<Transact, Err = Traced<database::Error>>,
    Transacted<Db>: Database<
            Select<By<Option<User>, user::Id>>,
            Ok = Option<User>,
            Err = Traced<database::Error>,
        > + Database<
            Lock<By<User, user::Id>>,
            Ok = (),
            Err = Traced<database::Error>,
        > + Database<Update<User>, Ok = (), Err = Traced<database::Error>>
        + Database<Commit, Ok = (), Err = Traced<database::Error>>,
{
    type Ok = User;
    type Err = Traced<ExecutionError>;

    async fn execute(
        &self,
        cmd: UpdateUserPhone,
    ) -> Result<Self::Ok, Self::Err> {
        use ExecutionError as E;

        let UpdateUserPhone { user_id, number } = cmd;

        let tx = self
            .database()
            .execute(Transact)
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;

        // Avoid concurrent actions upon the same `User`.
        tx.execute(Lock(By::new(user_id)))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))
            .map(drop)?;

        let mut user = tx
            .execute(Select(By::<Option<User>, _>::new(user_id)))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?
            .ok_or(E::UserNotExists(user_id))
            .map_err(tracerr::wrap!())?;
        if user.phone == number {
            return Ok(user);
        }

        user.phone = number;
        tx.execute(Update(user.clone()))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;

        tx.execute(Commit)
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;

        Ok(user)
    }
}

/// Error of [`UpdateUserPhone`] [`Command`] execution.
#[derive(Debug, Display, Error, From)]
pub enum ExecutionError {
    /// [`Database`] error.
    #[display("`Database` operation failed: {_0}")]
    Db(database::Error),

    /// [`User`] doesn't exist.
    #[display("`User(id: {_0}` does not exist")]
    #[from(ignore)]
    UserNotExists(#[error(not(source))] user::Id),
}
