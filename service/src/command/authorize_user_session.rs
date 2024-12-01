//! [`Command`] for authorizing a [`User`].

use common::operations::{By, Select};
use derive_more::{Display, Error, From};
use jsonwebtoken::Validation;
use tracerr::Traced;

use crate::{
    domain::{
        user::{self, session, Session},
        User,
    },
    infra::{database, Database},
    Service,
};

use super::Command;

/// [`Command`] for authorizing a [`User`].
#[derive(Clone, Debug, From)]
pub struct AuthorizeUserSession {
    /// [`Session`] token to authorize.
    pub token: session::Token,
}

impl<Db> Command<AuthorizeUserSession> for Service<Db>
where
    Db: Database<
        Select<By<Option<User>, user::Id>>,
        Ok = Option<User>,
        Err = Traced<database::Error>,
    >,
{
    type Ok = Session;
    type Err = Traced<ExecutionError>;

    async fn execute(
        &self,
        cmd: AuthorizeUserSession,
    ) -> Result<Self::Ok, Self::Err> {
        use ExecutionError as E;

        let AuthorizeUserSession { token } = cmd;

        let session = jsonwebtoken::decode::<Session>(
            token.as_ref(),
            &self.config.jwt_decoding_key,
            &Validation::default(),
        )
        .map_err(tracerr::from_and_wrap!(=> E))?
        .claims;

        drop(
            self.database()
                .execute(Select(By::new(session.user_id)))
                .await
                .map_err(tracerr::map_from_and_wrap!(=> E))?
                .ok_or_else(|| E::UserNotExists(session.user_id))
                .map_err(tracerr::wrap!())?,
        );

        Ok(session)
    }
}

/// Error of [`AuthorizeUserSession`] [`Command`] execution.
#[derive(Debug, Display, Error, From)]
pub enum ExecutionError {
    /// [`Database`] error.
    #[display("`Database` operation failed: {_0}")]
    Db(database::Error),

    /// [`jsonwebtoken`] decoding error.
    #[display("Failed to decode a JSON Web Token: {_0}")]
    JsonWebTokenDecodeError(jsonwebtoken::errors::Error),

    /// [`User`] the [`Session`] belongs to does not exist.
    #[display("`User(id: {_0}` does not exist")]
    #[from(ignore)]
    UserNotExists(#[error(not(source))] user::Id),
}
