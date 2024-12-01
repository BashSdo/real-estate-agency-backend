//! [`Command`] for creating a new [`User`].

use common::{
    operations::{By, Commit, Insert, Select, Transact, Transacted},
    DateTime,
};
use derive_more::{Display, Error, From};
use secrecy::{ExposeSecret, SecretBox};
use tracerr::Traced;

#[cfg(doc)]
use crate::domain::user::{Email, Login, Name, Password, Phone};
use crate::{
    domain::{user, User},
    infra::{database, Database},
    Service,
};

use super::Command;

/// [`Command`] for creating a new [`User`].
#[derive(Clone, Debug)]
pub struct CreateUser {
    /// [`Name`] of a new [`User`].
    pub name: user::Name,

    /// [`Login`] of a new [`User`].
    pub login: user::Login,

    /// [`Password`] of a new [`User`].
    pub password: SecretBox<user::Password>,

    /// [`Email`] of a new [`User`].
    pub email: Option<user::Email>,

    /// [`Phone`] of a new [`User`].
    pub phone: Option<user::Phone>,
}

impl<Db> Command<CreateUser> for Service<Db>
where
    Db: for<'l> Database<
            Select<By<Option<User>, &'l user::Login>>,
            Ok = Option<User>,
            Err = Traced<database::Error>,
        > + Database<Transact, Err = Traced<database::Error>>,
    Transacted<Db>: Database<Insert<User>, Err = Traced<database::Error>>
        + Database<Commit, Err = Traced<database::Error>>,
{
    type Ok = User;
    type Err = Traced<ExecutionError>;

    async fn execute(&self, cmd: CreateUser) -> Result<Self::Ok, Self::Err> {
        use ExecutionError as E;

        let CreateUser {
            name,
            login,
            password,
            email,
            phone,
        } = cmd;

        if email.is_none() && phone.is_none() {
            return Err(tracerr::new!(E::NoContactInfo));
        }

        let u = self
            .database()
            .execute(Select(By::new(&login)))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;
        if u.is_some() {
            return Err(tracerr::new!(E::LoginOccupied(login)));
        }

        let user = User {
            id: user::Id::new(),
            name,
            login,
            password_hash: user::PasswordHash::new(password.expose_secret()),
            email,
            phone,
            created_at: DateTime::now().coerce(),
            deleted_at: None,
        };

        let tx = self
            .database()
            .execute(Transact)
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;
        tx.execute(Insert(user.clone()))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))
            .map(drop)?;
        tx.execute(Commit)
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))
            .map(drop)?;

        Ok(user)
    }
}

/// Error of [`CreateUser`] [`Command`] execution.
#[derive(Debug, Display, Error, From)]
pub enum ExecutionError {
    /// [`Database`] error.
    #[display("`Database` operation failed: {_0}")]
    #[from]
    Db(database::Error),

    /// [`user::Login`] is already occupied.
    #[display("`{_0}` login is occupied")]
    LoginOccupied(#[error(not(source))] user::Login),

    /// No contact information provided.
    #[display("No contact information provided")]
    NoContactInfo,
}
