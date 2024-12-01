//! [`Command`] for creating a new [`contract::ManagementForSale`].

use std::collections::HashMap;

use common::{
    operations::{By, Commit, Insert, Lock, Select, Transact, Transacted},
    DateTime, Money, Percent,
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

/// [`Command`] for creating a new [`contract::ManagementForSale`].
#[derive(Clone, Debug)]
pub struct CreateManagementForSaleContract {
    /// ID of the [`Realty`] to manage.
    pub realty_id: realty::Id,

    /// ID of the [`User`] to owns the [`Realty`].
    pub landlord_id: user::Id,

    /// ID of the [`User`] who will manage the [`Realty`].
    pub employer_id: user::Id,

    /// Name of a new [`Contract`].
    pub name: contract::Name,

    /// Description of a new [`Contract`].
    pub description: contract::Description,

    /// [`DateTime`] when a new [`Contract`] expires.
    pub expires_at: Option<contract::ExpirationDateTime>,

    /// Expected price for sale a [`Realty`].
    pub expected_price: Money,

    /// Expected deposit to be paid before selling a [`Realty`].
    pub expected_deposit: Option<Money>,

    /// One-time fee for a [`Realty`] management.
    pub one_time_fee: Option<Money>,

    /// Monthly fee for a [`Realty`] management.
    pub monthly_fee: Option<Money>,

    /// Percent fee for a [`Realty`] management.
    pub percent_fee: Option<Percent>,

    /// Indicator whether [`Placement`] should be created for the [`Realty`].
    pub make_placement: bool,
}

impl<Db> Command<CreateManagementForSaleContract> for Service<Db>
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
            Select<By<Option<Active<contract::ManagementForSale>>, realty::Id>>,
            Ok = Option<Active<contract::ManagementForSale>>,
            Err = Traced<database::Error>,
        > + Database<Insert<Contract>, Err = Traced<database::Error>>
        + Database<Lock<By<Realty, realty::Id>>, Err = Traced<database::Error>>
        + Database<Commit, Err = Traced<database::Error>>,
{
    type Ok = Contract;
    type Err = Traced<ExecutionError>;

    async fn execute(
        &self,
        cmd: CreateManagementForSaleContract,
    ) -> Result<Self::Ok, Self::Err> {
        use ExecutionError as E;

        let CreateManagementForSaleContract {
            realty_id,
            landlord_id,
            employer_id,
            name,
            description,
            expires_at,
            expected_price,
            expected_deposit,
            one_time_fee,
            monthly_fee,
            percent_fee,
            make_placement,
        } = cmd;

        let realty = self
            .database()
            .execute(Select(By::<Option<Realty>, _>::new(realty_id)))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?
            .ok_or(E::RealtyNotExists(realty_id))
            .map_err(tracerr::wrap!())?;

        let users = self
            .database()
            .execute(Select(By::new([landlord_id, employer_id])))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?;
        let landlord = users
            .get(&landlord_id)
            .ok_or(E::UserNotExists(landlord_id))
            .map_err(tracerr::wrap!())?;
        let employer = users
            .get(&employer_id)
            .ok_or(E::UserNotExists(employer_id))
            .map_err(tracerr::wrap!())?;

        self.database()
            .execute(Select(
                By::<Option<Active<contract::Employment>>, _>::new(employer.id),
            ))
            .await
            .map_err(tracerr::map_from_and_wrap!(=> E))?
            .ok_or(E::UserNotEmployer(employer.id))
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

        let realty_contract =
            tx.execute(Select(By::<
                Option<Active<contract::ManagementForSale>>,
                _,
            >::new(realty.id)))
                .await
                .map_err(tracerr::map_from_and_wrap!(=> E))?;
        if realty_contract.is_some() {
            return Err(tracerr::new!(E::RealtyAlreadyManaged(realty.id)));
        }

        let contract = Contract::from(contract::ManagementForSale {
            id: contract::Id::new(),
            name,
            description,
            realty_id: realty.id,
            landlord_id: landlord.id,
            employer_id: employer.id,
            expected_price,
            expected_deposit,
            one_time_fee,
            monthly_fee,
            percent_fee,
            is_placed: make_placement,
            created_at: DateTime::now().coerce(),
            expires_at,
            terminated_at: None,
        });
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

/// Error of [`CreateManagementForSaleContract`] [`Command`] execution.
#[derive(Debug, Display, Error, From)]
pub enum ExecutionError {
    /// [`Database`] error.
    #[display("`Database` operation failed: {_0}")]
    #[from]
    Db(database::Error),

    /// [`Realty`] is already managed.
    #[display("`Realty(id: {_0})` is already managed")]
    RealtyAlreadyManaged(#[error(not(source))] realty::Id),

    /// [`Realty`] with the provided ID does not exist.
    #[display("`Realty(id: {_0})` does not exist")]
    RealtyNotExists(#[error(not(source))] realty::Id),

    /// [`User`] with the provided ID does not exist.
    #[display("`User(id: {_0})` does not exist")]
    UserNotExists(#[error(not(source))] user::Id),

    /// [`User`] is not an employer.
    #[display("`User(id: {_0})` is not an employer")]
    UserNotEmployer(#[error(not(source))] user::Id),
}
