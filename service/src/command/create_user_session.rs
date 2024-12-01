//! [`Command`] for creating a [`Session`].

use std::time::Duration;

use common::{
    operations::{By, Select},
    DateTime,
};
use derive_more::{Display, Error, From};
use secrecy::{ExposeSecret, SecretBox};
use tracerr::Traced;

#[cfg(doc)]
use crate::domain::user::{session::Token, Login, Password};
use crate::{
    domain::{
        user::{self, session, Session},
        User,
    },
    infra::{database, Database},
    Service,
};

use super::Command;

/// [`Command`] for creating a [`Session`].
#[derive(Clone, Debug, From)]
pub enum CreateUserSession {
    /// Create a new [`Session`] by [`User`] credentials.
    ByCredentials {
        /// [`Login`] of a [`User`].
        login: user::Login,

        /// [`Password`] of a [`User`].
        password: SecretBox<user::Password>,
    },

    /// Create a new [`Session`] by [`User`] ID.
    ByUserId(user::Id),
}

impl CreateUserSession {
    /// [`Duration`] of [`Session`] expiration.
    const EXPIRATION_DURATION: Duration = Duration::from_secs(30 * 60);
}

/// Output of [`CreateUserSession`] [`Command`].
#[derive(Clone, Debug)]
pub struct Output {
    /// [`Token`] of the created [`Session`].
    pub token: session::Token,

    /// [`User`] whose [`Session`] has been created.
    pub user: User,

    /// [`DateTime`] when the [`Session`] expires.
    pub expires_at: session::ExpirationDateTime,
}

impl<Db> Command<CreateUserSession> for Service<Db>
where
    Db: Database<
            Select<By<Option<User>, user::Id>>,
            Ok = Option<User>,
            Err = Traced<database::Error>,
        > + for<'l> Database<
            Select<By<Option<User>, &'l user::Login>>,
            Ok = Option<User>,
            Err = Traced<database::Error>,
        >,
{
    type Ok = Output;
    type Err = Traced<ExecutionError>;

    async fn execute(
        &self,
        cmd: CreateUserSession,
    ) -> Result<Self::Ok, Self::Err> {
        use CreateUserSession as Cmd;
        use ExecutionError as E;

        let user = match cmd {
            Cmd::ByCredentials { login, password } => {
                let user = self
                    .database()
                    .execute(Select(By::new(&login)))
                    .await
                    .map_err(tracerr::map_from_and_wrap!(=> E))?
                    .ok_or_else(|| E::WrongCredentials)
                    .map_err(tracerr::wrap!())?;

                let hash = user::PasswordHash::new(password.expose_secret());
                if user.password_hash != hash {
                    return Err(tracerr::new!(E::WrongCredentials));
                }

                user
            }
            Cmd::ByUserId(user_id) => self
                .database()
                .execute(Select(By::new(user_id)))
                .await
                .map_err(tracerr::map_from_and_wrap!(=> E))?
                .ok_or_else(|| E::UserNotExists(user_id))
                .map_err(tracerr::wrap!())?,
        };

        let expires_at = (DateTime::now() + Cmd::EXPIRATION_DURATION).coerce();
        let token = jsonwebtoken::encode::<Session>(
            &jsonwebtoken::Header::default(),
            &Session {
                user_id: user.id,
                expires_at,
            },
            &self.config.jwt_encoding_key,
        )
        .map_err(tracerr::from_and_wrap!(=> E))?;

        // SAFETY: `jsonwebtoken::encode` always returns a valid
        //         `session::Token`.
        #[expect(unsafe_code, reason = "invariants are preserved")]
        let token = unsafe { session::Token::new_unchecked(token) };

        Ok(Output {
            token,
            user,
            expires_at,
        })
    }
}

/// Error of [`CreateUserSession`] [`Command`] execution.
#[derive(Debug, Display, Error, From)]
pub enum ExecutionError {
    /// [`Database`] error.
    #[display("`Database` operation failed: {_0}")]
    Db(database::Error),

    /// [`jsonwebtoken`] encoding error.
    #[display("Failed to encode a JSON Web Token: {_0}")]
    JsonWebTokenEncodeError(jsonwebtoken::errors::Error),

    /// [`User`] with the provided ID does not exist.
    #[display("`User(id: {_0}` does not exist")]
    #[from(ignore)]
    UserNotExists(#[error(not(source))] user::Id),

    /// [`CreateUserSession::ByCredentials`] contains wrong credentials.
    #[display("Wrong `User` credentials")]
    WrongCredentials,
}
