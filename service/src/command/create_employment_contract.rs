//! [`Command`] for creating a new [`contract::Employment`].

use std::collections::HashMap;

use common::{
    operations::{By, Commit, Insert, Select, Transact, Transacted},
    DateTime, Money,
};
use derive_more::{Display, Error, From};
use tracerr::Traced;

use crate::{
    domain::{contract, user, Contract, User},
    infra::{database, Database},
    read::contract::Active,
    Service,
};

use super::Command;

/// [`Command`] for creating a new [`contract::Employment`].
#[derive(Clone, Debug)]
pub struct CreateEmploymentContract {
    /// ID of the [`User`] to be employed.
    pub user_id: user::Id,

    /// ID of the [`User`] who hires.
    pub initiator_id: user::Id,

    /// Name of a new [`Contract`].
    pub name: contract::Name,

    /// Description of a new [`Contract`].
    pub description: contract::Description,

    /// [`DateTime`] when a new [`Contract`] expires.
    pub expires_at: Option<contract::ExpirationDateTime>,

    /// Base salary of a new employer.
    pub base_salary: Money,
}

impl<Db> Command<CreateEmploymentContract> for Service<Db>
where
    Db: Database<Transact, Err = Traced<database::Error>>
        + Database<
            Select<By<HashMap<user::Id, User>, [user::Id; 2]>>,
            Ok = HashMap<user::Id, User>,
            Err = Traced<database::Error>,
        > + Database<
            Select<By<Option<Active<contract::Employment>>, user::Id>>,
            Ok = Option<Active<contract::Employment>>,
            Err = Traced<database::Error>,
        >,
    Transacted<Db>: Database<Insert<Contract>, Err = Traced<database::Error>>
        + Database<Commit, Err = Traced<database::Error>>,
{
    type Ok = Contract;
    type Err = Traced<ExecutionError>;

    async fn execute(
        &self,
        cmd: CreateEmploymentContract,
    ) -> Result<Self::Ok, Self::Err> {
        use ExecutionError as E;

        let CreateEmploymentContract {
            user_id,
            initiator_id,
            name,
            description,
            expires_at,
            base_salary,
        } = cmd;

        let users = self
            .database()
            .execute(Select(By::new([user_id, initiator_id])))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;
        let user = users
            .get(&user_id)
            .ok_or(E::UserNotExists(user_id))
            .map_err(tracerr::wrap!())?;
        let initiator = users
            .get(&initiator_id)
            .ok_or(E::UserNotExists(initiator_id))
            .map_err(tracerr::wrap!())?;

        self.database()
            .execute(Select(
                By::<Option<Active<contract::Employment>>, _>::new(
                    initiator.id,
                ),
            ))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?
            .ok_or(E::UserNotEmployer(initiator.id))
            .map_err(tracerr::wrap!())
            .map(drop)?;

        let existing_contract = self
            .database()
            .execute(Select(
                By::<Option<Active<contract::Employment>>, _>::new(user.id),
            ))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;
        if existing_contract.is_some() {
            return Err(tracerr::new!(E::UserAlreadyEmployed(user.id)));
        }

        let contract = Contract::from(contract::Employment {
            id: contract::Id::new(),
            name,
            description,
            employer_id: user.id,
            base_salary,
            created_at: DateTime::now().coerce(),
            expires_at,
            terminated_at: None,
        });

        let tx = self
            .database()
            .execute(Transact)
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;
        tx.execute(Insert(contract.clone()))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))
            .map(drop)?;
        tx.execute(Commit)
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))
            .map(drop)?;

        Ok(contract)
    }
}

/// Error of [`CreateEmploymentContract`] [`Command`] execution.
#[derive(Debug, Display, Error, From)]
pub enum ExecutionError {
    /// [`Database`] error.
    #[display("`Database` operation failed: {_0}")]
    #[from]
    Db(database::Error),

    /// [`User`] is already employed.
    #[display("`User(id: {_0})` is already employed")]
    UserAlreadyEmployed(#[error(not(source))] user::Id),

    /// [`User`] is not an employer.
    #[display("`User(id: {_0})` is not an employer")]
    UserNotEmployer(#[error(not(source))] user::Id),

    /// [`User`] with the provided ID does not exist.
    #[display("`User(id: {_0})` does not exist")]
    UserNotExists(#[error(not(source))] user::Id),
}
