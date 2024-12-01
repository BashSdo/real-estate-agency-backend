//! [`Command`] for updating an [`user::Email`].

use common::operations::{
    By, Commit, Lock, Select, Transact, Transacted, Update,
};
use derive_more::{Display, Error, From};
use tracerr::Traced;

#[cfg(doc)]
use crate::domain::user::Email;
use crate::{
    domain::{user, User},
    infra::{database, Database},
    Service,
};

use super::Command;

/// [`Command`] for updating an [`user::Email`].
#[derive(Clone, Debug, From)]
pub struct UpdateUserEmail {
    /// ID of the [`User`] which [`Email`] should be updated.
    pub user_id: user::Id,

    /// New [`Email`] address of the [`User`].
    ///
    /// [`None`] indicating [`Email`] deletion.
    pub address: Option<user::Email>,
}

impl<Db> Command<UpdateUserEmail> for Service<Db>
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
        cmd: UpdateUserEmail,
    ) -> Result<Self::Ok, Self::Err> {
        use ExecutionError as E;

        let UpdateUserEmail { user_id, address } = cmd;

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
        if user.email == address {
            return Ok(user);
        }

        user.email = address;
        tx.execute(Update(user.clone()))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;

        tx.execute(Commit)
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;

        Ok(user)
    }
}

/// Error of [`UpdateUserEmail`] [`Command`] execution.
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
