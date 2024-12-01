//! [`Command`] for updating an [`user::Password`].

use common::operations::{
    By, Commit, Lock, Select, Transact, Transacted, Update,
};
use derive_more::{Display, Error, From};
use tracerr::Traced;

#[cfg(doc)]
use crate::domain::user::Password;
use crate::{
    domain::{user, User},
    infra::{database, Database},
    Service,
};

use super::Command;

/// [`Command`] for updating an [`user::Password`].
#[derive(Clone, Debug, From)]
pub struct UpdateUserPassword {
    /// ID of the [`User`] which [`Password`] should be updated.
    pub user_id: user::Id,

    /// New [`Password`] of the [`User`].
    pub new_password: user::Password,

    /// Old [`Password`] of the [`User`].
    pub old_password: user::Password,
}

impl<Db> Command<UpdateUserPassword> for Service<Db>
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
        cmd: UpdateUserPassword,
    ) -> Result<Self::Ok, Self::Err> {
        use ExecutionError as E;

        let UpdateUserPassword {
            user_id,
            new_password,
            old_password,
        } = cmd;

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

        let new_password_hash = user::PasswordHash::new(&new_password);
        let old_password_hash = user::PasswordHash::new(&old_password);

        let mut user = tx
            .execute(Select(By::<Option<User>, _>::new(user_id)))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?
            .ok_or(E::UserNotExists(user_id))
            .map_err(tracerr::wrap!())?;
        if user.password_hash != old_password_hash {
            return Err(tracerr::new!(E::WrongPassword));
        }

        if user.password_hash == new_password_hash {
            return Ok(user);
        }

        user.password_hash = new_password_hash;
        tx.execute(Update(user.clone()))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;

        tx.execute(Commit)
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;

        Ok(user)
    }
}

/// Error of [`UpdateUserPassword`] [`Command`] execution.
#[derive(Debug, Display, Error, From)]
pub enum ExecutionError {
    /// [`Database`] error.
    #[display("`Database` operation failed: {_0}")]
    Db(database::Error),

    /// [`User`] doesn't exist.
    #[display("`User(id: {_0}` does not exist")]
    #[from(ignore)]
    UserNotExists(#[error(not(source))] user::Id),

    /// Wrong old [`Password`] provided.
    #[display("Wrong old password")]
    WrongPassword,
}
