//! [`Command`] for deplacing a [`Contract`] as [`Placement`].

use common::operations::{
    By, Commit, Insert, Lock, Select, Transact, Transacted,
};
use derive_more::{Display, Error, From};
use tracerr::Traced;

#[cfg(doc)]
use crate::read::Placement;
use crate::{
    domain::{contract, realty, user, Contract, Realty, User},
    infra::{database, Database},
    read::contract::Active,
    Service,
};

use super::Command;

/// [`Command`] for deplacing a [`Contract`] as [`Placement`].
#[derive(Clone, Copy, Debug)]
pub struct DeplaceContract {
    /// ID of the [`Contract`] to be deplaced.
    pub contract_id: contract::Id,

    /// ID of the [`User`] who deplaces the [`Contract`].
    pub initiator_id: user::Id,
}

impl<Db> Command<DeplaceContract> for Service<Db>
where
    Db: Database<Transact, Err = Traced<database::Error>>
        + Database<
            Select<By<Option<User>, user::Id>>,
            Ok = Option<User>,
            Err = Traced<database::Error>,
        > + Database<
            Select<By<Option<Active<contract::Employment>>, user::Id>>,
            Ok = Option<Active<contract::Employment>>,
            Err = Traced<database::Error>,
        > + Database<
            Select<By<Option<Contract>, contract::Id>>,
            Ok = Option<Contract>,
            Err = Traced<database::Error>,
        >,
    Transacted<Db>: Database<
            Lock<By<Contract, contract::Id>>,
            Err = Traced<database::Error>,
        > + Database<
            Select<By<Option<Contract>, contract::Id>>,
            Ok = Option<Contract>,
            Err = Traced<database::Error>,
        > + Database<Insert<Contract>, Err = Traced<database::Error>>
        + Database<Commit, Err = Traced<database::Error>>,
    Transacted<Db>:
        Database<Lock<By<Realty, realty::Id>>, Err = Traced<database::Error>>,
{
    type Ok = Contract;
    type Err = Traced<ExecutionError>;

    async fn execute(
        &self,
        cmd: DeplaceContract,
    ) -> Result<Self::Ok, Self::Err> {
        use ExecutionError as E;

        let DeplaceContract {
            contract_id,
            initiator_id,
        } = cmd;

        let initiator = self
            .database()
            .execute(Select(By::<Option<User>, _>::new(initiator_id)))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?
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

        let contract = self
            .database()
            .execute(Select(By::<Option<Contract>, _>::new(contract_id)))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?
            .filter(Contract::is_active)
            .ok_or(E::ContractNotExists(contract_id))
            .map_err(tracerr::wrap!())?;

        let tx = self
            .database()
            .execute(Transact)
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;

        if let Some(realty_id) = contract.realty_id() {
            // Avoid concurrent actions upon the same `Realty`.
            tx.execute(Lock(By::new(realty_id)))
                .await
                .map_err(tracerr::map_from_and_wrap!(=> E))
                .map(drop)?;
        }

        // Avoid concurrent deletions.
        tx.execute(Lock(By::new(contract.id())))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))
            .map(drop)?;

        let mut contract = tx
            .execute(Select(By::<Option<Contract>, _>::new(contract_id)))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?
            .filter(Contract::is_active)
            .ok_or(E::ContractNotExists(contract_id))
            .map_err(tracerr::wrap!())?;

        if let Some(is_placed) = contract.is_placed_mut() {
            if !*is_placed {
                return Err(tracerr::new!(E::ContractNotPlaced(contract_id)));
            }

            *is_placed = false;
        } else {
            return Err(tracerr::new!(E::UnsupportedContract(contract_id)));
        }

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

/// Error of [`DeplaceContract`] [`Command`] execution.
#[derive(Debug, Display, Error, From)]
pub enum ExecutionError {
    /// [`Contract`] with the provided ID does not exist.
    #[display("`Contract(id: {_0})` does not exist")]
    ContractNotExists(#[error(not(source))] contract::Id),

    /// [`Contract`] is not placed.
    #[display("`Contract(id: {_0})` is not placed")]
    ContractNotPlaced(#[error(not(source))] contract::Id),

    /// [`Database`] error.
    #[display("`Database` operation failed: {_0}")]
    #[from]
    Db(database::Error),

    /// Unsupported [`Contract`].
    #[display("`Contract(id: {_0})` is not supported")]
    UnsupportedContract(#[error(not(source))] contract::Id),

    /// [`User`] is not an employer.
    #[display("`User(id: {_0})` is not an employer")]
    UserNotEmployer(#[error(not(source))] user::Id),

    /// [`User`] with the provided ID does not exist.
    #[display("`User(id: {_0})` does not exist")]
    UserNotExists(#[error(not(source))] user::Id),
}
