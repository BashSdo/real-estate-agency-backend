//! [`Command`] for creating a new [`contract::Sale`].

use std::collections::HashMap;

use common::{
    operations::{
        By, Commit, Insert, Lock, Select, Transact, Transacted, Update,
    },
    DateTime, Money,
};
use derive_more::{Display, Error, From};
use tracerr::Traced;

use crate::{
    domain::{contract, realty, user, Contract, Realty, User},
    infra::{database, Database},
    read::{self, contract::Active},
    Service,
};

use super::Command;

/// [`Command`] for creating a new [`contract::Sale`].
#[derive(Clone, Debug)]
pub struct CreateSaleContract {
    /// ID of the [`Realty`] to manage.
    pub realty_id: realty::Id,

    /// ID of the [`User`] who will manage the [`Realty`].
    pub employer_id: user::Id,

    /// ID of the [`User`] who rents the [`Realty`].
    pub purchaser_id: user::Id,

    /// Name of a new [`Contract`].
    pub name: contract::Name,

    /// Description of a new [`Contract`].
    pub description: contract::Description,

    /// [`DateTime`] when a new [`Contract`] expires.
    pub expires_at: Option<contract::ExpirationDateTime>,

    /// Monthly price for rent a [`Realty`].
    pub price: Money,

    /// Deposit to be paid at the beginning of the [`Realty`] rent.
    pub deposit: Option<Money>,
}

impl<Db> Command<CreateSaleContract> for Service<Db>
where
    Db: Database<Transact, Err = Traced<database::Error>>
        + Database<
            Select<By<HashMap<user::Id, User>, [user::Id; 2]>>,
            Ok = HashMap<user::Id, User>,
            Err = Traced<database::Error>,
        > + Database<
            Select<By<Option<Realty>, realty::Id>>,
            Ok = Option<Realty>,
            Err = Traced<database::Error>,
        > + Database<
            Select<By<Option<Active<contract::Employment>>, user::Id>>,
            Ok = Option<Active<contract::Employment>>,
            Err = Traced<database::Error>,
        >,
    Transacted<Db>: Database<
            Select<By<Option<Active<contract::ManagementForRent>>, realty::Id>>,
            Ok = Option<Active<contract::ManagementForRent>>,
            Err = Traced<database::Error>,
        > + Database<
            Select<By<Option<Active<contract::ManagementForSale>>, realty::Id>>,
            Ok = Option<Active<contract::ManagementForSale>>,
            Err = Traced<database::Error>,
        > + Database<
            Select<By<read::realty::IsRented, realty::Id>>,
            Ok = read::realty::IsRented,
            Err = Traced<database::Error>,
        > + Database<Insert<Contract>, Err = Traced<database::Error>>
        + Database<Update<Contract>, Err = Traced<database::Error>>
        + Database<Commit, Err = Traced<database::Error>>,
    Transacted<Db>:
        Database<Lock<By<Realty, realty::Id>>, Err = Traced<database::Error>>,
{
    type Ok = Contract;
    type Err = Traced<ExecutionError>;

    #[expect(clippy::too_many_lines, reason = "still readable")]
    async fn execute(
        &self,
        cmd: CreateSaleContract,
    ) -> Result<Self::Ok, Self::Err> {
        use ExecutionError as E;

        let CreateSaleContract {
            realty_id,
            employer_id,
            purchaser_id,
            name,
            description,
            expires_at,
            price,
            deposit,
        } = cmd;

        let realty = self
            .database()
            .execute(Select(By::<Option<Realty>, _>::new(realty_id)))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?
            .ok_or(E::RealtyNotManaged(realty_id))
            .map_err(tracerr::wrap!())?;

        let users = self
            .database()
            .execute(Select(By::new([employer_id, purchaser_id])))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;
        let employer = users
            .get(&employer_id)
            .ok_or(E::UserNotExists(employer_id))
            .map_err(tracerr::wrap!())?;
        let purchaser = users
            .get(&purchaser_id)
            .ok_or(E::UserNotExists(purchaser_id))
            .map_err(tracerr::wrap!())?;

        self.database()
            .execute(Select(
                By::<Option<Active<contract::Employment>>, _>::new(employer_id),
            ))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?
            .ok_or(E::UserNotEmployer(employer_id))
            .map_err(tracerr::wrap!())
            .map(drop)?;

        let tx = self
            .database()
            .execute(Transact)
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;

        // Avoid concurrent actions upon the same `Realty`.
        tx.execute(Lock(By::new(realty.id)))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))
            .map(drop)?;

        let Active(mut realty_contract) =
            tx.execute(Select(By::<
                Option<Active<contract::ManagementForSale>>,
                _,
            >::new(realty.id)))
                .await
                .map_err(tracerr::map_from_and_wrap!(=> E))?
                .ok_or(E::RealtyNotManaged(realty.id))
                .map_err(tracerr::wrap!())?;
        if realty_contract.employer_id != employer_id {
            // TODO: Reconsider this.
            return Err(tracerr::new!(E::UserNotManager(employer_id)));
        }

        let managed_for_rent_contract =
            tx.execute(Select(By::<
                Option<Active<contract::ManagementForRent>>,
                _,
            >::new(realty.id)))
                .await
                .map_err(tracerr::map_from_and_wrap!(=> E))?;
        if managed_for_rent_contract.is_some() {
            return Err(tracerr::new!(E::RealtyManagedForRent(realty.id)));
        }

        let is_rented = tx
            .execute(Select(By::<read::realty::IsRented, _>::new(realty.id)))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;
        if *is_rented {
            return Err(tracerr::new!(E::RealtyRented(realty.id)));
        }

        let contract = Contract::from(contract::Rent {
            id: contract::Id::new(),
            name,
            description,
            realty_id: realty.id,
            purchaser_id: purchaser.id,
            // Landlord existence is guaranteed by the
            // `contract::ManagementForSale` existence.
            landlord_id: realty_contract.landlord_id,
            employer_id: employer.id,
            price,
            deposit,
            created_at: DateTime::now().coerce(),
            expires_at,
            terminated_at: None,
        });
        tx.execute(Insert(contract.clone()))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))
            .map(drop)?;

        realty_contract.terminated_at = Some(DateTime::now().coerce());
        tx.execute(Update(realty_contract.into()))
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

/// Error of [`CreateSaleContract`] [`Command`] execution.
#[derive(Debug, Display, Error, From)]
pub enum ExecutionError {
    /// [`Database`] error.
    #[display("`Database` operation failed: {_0}")]
    #[from]
    Db(database::Error),

    /// [`Realty`] with the provided ID is managed for rent.
    #[display("`Realty(id: {_0})` is managed for rent")]
    RealtyManagedForRent(#[error(not(source))] realty::Id),

    /// [`Realty`] with the provided ID doesn't have a
    /// [`contract::ManagementForSale`].
    #[display(
        "`Realty(id: {_0})` doesn't have a `contract::ManagementForSale`"
    )]
    RealtyNotManaged(#[error(not(source))] realty::Id),

    /// [`Realty`] with the provided ID is rented.
    #[display("`Realty(id: {_0})` is rented")]
    RealtyRented(#[error(not(source))] realty::Id),

    /// [`User`] is not an employer.
    #[display("`User(id: {_0})` is not an employer")]
    UserNotEmployer(#[error(not(source))] user::Id),

    /// [`User`] with the provided ID does not exist.
    #[display("`User(id: {_0})` does not exist")]
    UserNotExists(#[error(not(source))] user::Id),

    /// [`User`] is not a manager of the [`Realty`].
    #[display("`User(id: {_0})` is not a manager of the `Realty`")]
    UserNotManager(#[error(not(source))] user::Id),
}
