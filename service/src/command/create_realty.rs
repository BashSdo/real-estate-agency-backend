//! [`Command`] for creating a new [`Realty`].

use common::{
    operations::{By, Commit, Insert, Lock, Select, Transact, Transacted},
    DateTime,
};
use tracerr::Traced;

#[cfg(doc)]
use crate::domain::realty::{
    ApartmentNum, BuildingName, City, Country, Floor, NumFloors, RoomNum,
    State, Street, ZipCode,
};
use crate::{
    domain::{realty, Realty},
    infra::{database, Database},
    Service,
};

use super::Command;

/// [`Command`] for creating a new [`Realty`].
#[derive(Clone, Debug)]
pub struct CreateRealty {
    /// [`Country`] of a new [`Realty`].
    pub country: realty::Country,

    /// [`State`] of a new [`Realty`].
    pub state: Option<realty::State>,

    /// [`City`] of a new [`Realty`].
    pub city: realty::City,

    /// [`Street`] of a new [`Realty`].
    pub street: realty::Street,

    /// [`ZipCode`] of a new [`Realty`].
    pub zip_code: Option<realty::ZipCode>,

    /// [`BuildingName`] of a new [`Realty`].
    pub building_name: realty::BuildingName,

    /// [`NumFloors`] of a new [`Realty`].
    pub num_floors: realty::NumFloors,

    /// [`Floor`] of a new [`Realty`].
    pub floor: Option<realty::Floor>,

    /// [`ApartmentNum`] of a new [`Realty`].
    pub apartment_num: Option<realty::ApartmentNum>,

    /// [`RoomNum`] of a new [`Realty`].
    pub room_num: Option<realty::RoomNum>,
}

impl<Db> Command<CreateRealty> for Service<Db>
where
    Db: Database<Transact, Err = Traced<database::Error>>,
    Transacted<Db>: Database<
            Select<By<Option<Realty>, realty::Hash>>,
            Ok = Option<Realty>,
            Err = Traced<database::Error>,
        > + Database<Insert<Realty>, Err = Traced<database::Error>>
        + Database<Commit, Err = Traced<database::Error>>,
    Transacted<Db>:
        Database<Lock<By<Realty, realty::Hash>>, Err = Traced<database::Error>>,
{
    type Ok = Realty;
    type Err = Traced<ExecutionError>;

    async fn execute(&self, cmd: CreateRealty) -> Result<Self::Ok, Self::Err> {
        let CreateRealty {
            country,
            state,
            city,
            street,
            zip_code,
            building_name,
            num_floors,
            floor,
            apartment_num,
            room_num,
        } = cmd;

        let hash = realty::Hash::new(
            &country,
            state.as_ref(),
            &city,
            &street,
            zip_code.as_ref(),
            &building_name,
            num_floors,
            floor,
            apartment_num.as_ref(),
            room_num.as_ref(),
        );

        let realty = Realty {
            id: realty::Id::new(),
            hash,
            address: realty::Address::from_parts(
                &country,
                state.as_ref(),
                &city,
                &street,
                zip_code.as_ref(),
                &building_name,
                floor,
                apartment_num.as_ref(),
                room_num.as_ref(),
            ),
            country,
            state,
            city,
            street,
            zip_code,
            building_name,
            num_floors,
            floor,
            apartment_num,
            room_num,
            created_at: DateTime::now().coerce(),
            deleted_at: None,
        };

        let tx = self
            .database()
            .execute(Transact)
            .await
            .map_err(tracerr::wrap!())?;

        // Avoid concurrent creation of the same `Realty`.
        tx.execute(Lock(By::new(hash)))
            .await
            .map_err(tracerr::wrap!())
            .map(drop)?;

        let existing_realty = tx
            .execute(Select(By::new(hash)))
            .await
            .map_err(tracerr::wrap!())?;
        if let Some(realty) = existing_realty {
            // `Realty` with the same properties already exists.
            return Ok(realty);
        }

        tx.execute(Insert(realty.clone()))
            .await
            .map_err(tracerr::wrap!())
            .map(drop)?;
        tx.execute(Commit)
            .await
            .map_err(tracerr::wrap!())
            .map(drop)?;

        Ok(realty)
    }
}

/// Error of [`CreateRealty`] [`Command`] execution.
pub type ExecutionError = database::Error;
