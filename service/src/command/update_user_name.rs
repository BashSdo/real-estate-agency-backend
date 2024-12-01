//! [`Command`] for updating an [`user::Name`].

use common::operations::{
    By, Commit, Lock, Select, Transact, Transacted, Update,
};
use derive_more::{Display, Error, From};
use tracerr::Traced;

#[cfg(doc)]
use crate::domain::user::Name;
use crate::{
    domain::{user, User},
    infra::{database, Database},
    Service,
};

use super::Command;

/// [`Command`] for updating an [`user::Name`].
#[derive(Clone, Debug, From)]
pub struct UpdateUserName {
    /// ID of the [`User`] which [`Name`] should be updated.
    pub user_id: user::Id,

    /// New [`Name`] of the [`User`].
    pub name: user::Name,
}

impl<Db> Command<UpdateUserName> for Service<Db>
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
        cmd: UpdateUserName,
    ) -> Result<Self::Ok, Self::Err> {
        use ExecutionError as E;

        let UpdateUserName { user_id, name } = cmd;

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
        if user.name == name {
            return Ok(user);
        }

        user.name = name;
        tx.execute(Update(user.clone()))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;

        tx.execute(Commit)
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;

        Ok(user)
    }
}

/// Error of [`UpdateUserName`] [`Command`] execution.
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
