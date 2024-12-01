//! [`Realty`]-related [`Database`] implementations.

use std::collections::HashMap;

use common::operations::{By, Delete, Insert, Lock, Select, Update};
use itertools::Itertools as _;
use postgres_types::ToSql;
use tracerr::Traced;

use crate::{
    domain::{contract, realty, Realty},
    infra::{
        database::{
            self,
            postgres::{Connection, FuzzPattern},
            Postgres,
        },
        Database,
    },
    read,
};

impl<C, IDs> Database<Select<By<HashMap<realty::Id, Realty>, IDs>>>
    for Postgres<C>
where
    C: Connection,
    IDs: AsRef<[realty::Id]>,
{
    type Ok = HashMap<realty::Id, Realty>;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<By<HashMap<realty::Id, Realty>, IDs>>,
    ) -> Result<Self::Ok, Self::Err> {
        let ids = by.into_inner();
        // Avoid subtle change for SQL.
        let ids: &[realty::Id] = ids.as_ref();
        if ids.is_empty() {
            return Ok(HashMap::new());
        }
        let limit = i32::try_from(ids.len()).unwrap();

        const SQL: &str = "\
            SELECT id, hash, address, \
                   country, state, city, street, zip_code, building_name, \
                   num_floors, floor, \
                   apartment_num, room_num, \
                   created_at \
            FROM realties \
            WHERE id IN (SELECT unnest($1::UUID[]) LIMIT $2::INT4) \
            LIMIT $2::INT4";
        Ok(self
            .query(SQL, &[&ids, &limit])
            .await
            .map_err(tracerr::wrap!())?
            .into_iter()
            .map(|row| {
                let id = row.get("id");
                (
                    id,
                    Realty {
                        id,
                        hash: row.get("hash"),
                        address: row.get("address"),
                        country: row.get("country"),
                        state: row.get("state"),
                        city: row.get("city"),
                        street: row.get("street"),
                        zip_code: row.get("zip_code"),
                        building_name: row.get("building_name"),
                        num_floors: u16::try_from(
                            row.get::<_, i32>("num_floors"),
                        )
                        .expect("`num_floors` overflow"),
                        floor: row
                            .get::<_, Option<i32>>("floor")
                            .map(u16::try_from)
                            .transpose()
                            .expect("`floor` overflow"),
                        apartment_num: row.get("apartment_num"),
                        room_num: row.get("room_num"),
                        created_at: row.get("created_at"),
                        // OK, because `Realty` removed from database completely
                        // once deleted.
                        deleted_at: None,
                    },
                )
            })
            .collect())
    }
}

impl<C> Database<Select<By<Option<Realty>, realty::Id>>> for Postgres<C>
where
    C: Connection,
    Self: Database<
        Select<By<HashMap<realty::Id, Realty>, [realty::Id; 1]>>,
        Ok = HashMap<realty::Id, Realty>,
        Err = Traced<database::Error>,
    >,
{
    type Ok = Option<Realty>;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<By<Option<Realty>, realty::Id>>,
    ) -> Result<Self::Ok, Self::Err> {
        let id = by.into_inner();
        Ok(self
            .execute(Select(By::new([id])))
            .await
            .map_err(tracerr::wrap!())?
            .remove(&id))
    }
}

impl<C> Database<Select<By<Option<Realty>, realty::Hash>>> for Postgres<C>
where
    C: Connection,
    Self: Database<
        Select<By<Option<Realty>, realty::Id>>,
        Ok = Option<Realty>,
        Err = Traced<database::Error>,
    >,
{
    type Ok = Option<Realty>;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<By<Option<Realty>, realty::Hash>>,
    ) -> Result<Self::Ok, Self::Err> {
        // Avoid subtle change for SQL.
        let hash: realty::Hash = by.into_inner();

        const SQL: &str = "\
            SELECT id \
            FROM realties \
            WHERE hash = $1::UUID \
            LIMIT 1";
        let Some(row) = self
            .query_opt(SQL, &[&hash])
            .await
            .map_err(tracerr::wrap!())?
        else {
            return Ok(None);
        };

        self.execute(Select(By::new(row.get("id"))))
            .await
            .map_err(tracerr::wrap!())
    }
}

impl<C> Database<Insert<Realty>> for Postgres<C>
where
    C: Connection,
    Self: Database<Update<Realty>, Ok = (), Err = Traced<database::Error>>,
{
    type Ok = ();
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Insert(realty): Insert<Realty>,
    ) -> Result<Self::Ok, Self::Err> {
        self.execute(Update(realty)).await.map_err(tracerr::wrap!())
    }
}

impl<C> Database<Update<Realty>> for Postgres<C>
where
    C: Connection,
{
    type Ok = ();
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Update(realty): Update<Realty>,
    ) -> Result<Self::Ok, Self::Err> {
        let Realty {
            id,
            hash,
            address,
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
            created_at,
            deleted_at,
        } = realty;

        let num_floors = i32::from(num_floors);
        let floor = floor.map(i32::from);

        if deleted_at.is_some() {
            const SQL: &str = "\
                DELETE FROM realties \
                WHERE id = $1::UUID \
                LIMIT 1";
            return self
                .exec(SQL, &[&id])
                .await
                .map_err(tracerr::wrap!())
                .map(drop);
        }

        const SQL: &str = "\
            INSERT INTO realties (\
                id, hash, address, \
                country, state, city, street, zip_code, building_name, \
                num_floors, floor, \
                apartment_num, room_num, \
                created_at \
            ) VALUES (\
                $1::UUID, $2::UUID, $3::VARCHAR, \
                $4::VARCHAR, \
                $5::VARCHAR, \
                $6::VARCHAR, \
                $7::VARCHAR, \
                $8::VARCHAR, \
                $9::VARCHAR, \
                $10::INT4, $11::INT4, \
                $12::VARCHAR, $13::VARCHAR, \
                $14::TIMESTAMPTZ \
            ) \
            ON CONFLICT (id) DO UPDATE \
            SET hash = EXCLUDED.hash, \
                address = EXCLUDED.address, \
                country = EXCLUDED.country, \
                state = EXCLUDED.state, \
                city = EXCLUDED.city, \
                street = EXCLUDED.street, \
                zip_code = EXCLUDED.zip_code, \
                building_name = EXCLUDED.building_name, \
                num_floors = EXCLUDED.num_floors, \
                floor = EXCLUDED.floor, \
                apartment_num = EXCLUDED.apartment_num, \
                room_num = EXCLUDED.room_num, \
                created_at = EXCLUDED.created_at";
        self.exec(
            SQL,
            &[
                &id,
                &hash,
                &address,
                &country,
                &state,
                &city,
                &street,
                &zip_code,
                &building_name,
                &num_floors,
                &floor,
                &apartment_num,
                &room_num,
                &created_at,
            ],
        )
        .await
        .map_err(tracerr::wrap!())
        .map(drop)
    }
}

impl<C> Database<Lock<By<Realty, realty::Id>>> for Postgres<C>
where
    C: Connection,
{
    type Ok = ();
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Lock(by): Lock<By<Realty, realty::Id>>,
    ) -> Result<Self::Ok, Self::Err> {
        // Avoid subtle change for SQL.
        let id: realty::Id = by.into_inner();

        const SQL: &str = "\
            INSERT INTO realties_lock \
            VALUES ($1::UUID) \
            ON CONFLICT (id) DO NOTHING";
        self.query(SQL, &[&id])
            .await
            .map_err(tracerr::wrap!())
            .map(drop)
    }
}

impl<C> Database<Lock<By<Realty, realty::Hash>>> for Postgres<C>
where
    C: Connection,
{
    type Ok = ();
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Lock(by): Lock<By<Realty, realty::Hash>>,
    ) -> Result<Self::Ok, Self::Err> {
        // Avoid subtle change for SQL.
        let hash: realty::Hash = by.into_inner();

        const SQL: &str = "\
            INSERT INTO realties_creation_lock \
            VALUES ($1::UUID) \
            ON CONFLICT (hash) DO NOTHING";
        self.query(SQL, &[&hash])
            .await
            .map_err(tracerr::wrap!())
            .map(drop)
    }
}

impl<C> Database<Select<By<read::realty::IsRented, realty::Id>>> for Postgres<C>
where
    C: Connection,
{
    type Ok = read::realty::IsRented;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<By<read::realty::IsRented, realty::Id>>,
    ) -> Result<Self::Ok, Self::Err> {
        // Avoid subtle change for SQL.
        let realty_id: realty::Id = by.into_inner();

        const SQL: &str = "\
            SELECT id \
            FROM contracts \
            WHERE kind = $1::INT2 \
              AND realty_id = $2::UUID \
              AND terminated_at IS NULL \
              AND (expires_at IS NULL \
                   OR expires_at > NOW()) \
            LIMIT 1";
        self.query_opt(SQL, &[&contract::Kind::Rent, &realty_id])
            .await
            .map_err(tracerr::wrap!())
            .map(|r| read::realty::IsRented(r.is_some()))
    }
}

impl<C>
    Database<Select<By<read::realty::list::Page, read::realty::list::Selector>>>
    for Postgres<C>
where
    C: Connection,
{
    type Ok = read::realty::list::Page;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<
            By<read::realty::list::Page, read::realty::list::Selector>,
        >,
    ) -> Result<Self::Ok, Self::Err> {
        let read::realty::list::Selector {
            arguments,
            filter: read::realty::list::Filter { address },
        } = by.into_inner();

        let limit = i32::try_from(arguments.limit()).unwrap() + 1;

        let mut ps: Vec<&(dyn ToSql + Sync)> = vec![&limit];

        let cursor_idx = arguments.cursor().map(|c| {
            ps.push(c);
            ps.len()
        });
        let address_idx = address.as_ref().map(|n| {
            ps.push(n);
            ps.len()
        });

        let address_pattern =
            address.as_ref().map(|n| FuzzPattern::new(n.as_ref()));
        let address_pattern_idx = address_pattern.as_ref().map(|n| {
            ps.push(n);
            ps.len()
        });

        let sql = format!(
            "SELECT id \
             FROM realties \
             WHERE true \
                   {cursor} \
                   {address_filtering} \
             ORDER BY {address_ordering} \
                      id {order} \
             LIMIT $1::INT4",
            cursor = cursor_idx.into_iter().format_with("", |idx, f| {
                let op = arguments.kind().operator();
                f(&format_args!("AND id {op} ${idx}::UUID"))
            }),
            order = arguments.kind().order().sql(),
            address_filtering =
                address_pattern_idx.into_iter().format_with("", |idx, f| {
                    f(&format_args!(
                        "AND LOWER(address) SIMILAR TO LOWER(${idx}::VARCHAR)"
                    ))
                }),
            address_ordering =
                address_idx.into_iter().format_with("", |idx, f| {
                    let order = arguments.kind().order().sql();
                    f(&format_args!(
                        "LEVENSHTEIN(address, ${idx}::VARCHAR, 1, 1, 0) \
                         {order},"
                    ))
                })
        );
        let rows = self
            .query(&sql, ps.as_slice())
            .await
            .map_err(tracerr::wrap!())?;

        let has_more = rows.len() > arguments.limit();
        let edges = rows
            .into_iter()
            .take(arguments.limit())
            .map(|row| {
                let id = row.get("id");
                (id, id)
            })
            .collect::<Vec<_>>();

        Ok(read::realty::list::Page::new(&arguments, edges, has_more))
    }
}

impl<C> Database<Select<By<read::realty::list::TotalCount, ()>>> for Postgres<C>
where
    C: Connection,
{
    type Ok = read::realty::list::TotalCount;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(_): Select<By<read::realty::list::TotalCount, ()>>,
    ) -> Result<Self::Ok, Self::Err> {
        const SQL: &str = "\
            SELECT COUNT(*)::INT4 \
            FROM realties";
        self.query_opt(SQL, &[])
            .await
            .map_err(tracerr::wrap!())
            .map(|row| row.expect("always exists").get::<_, i32>(0).into())
    }
}

impl<C> Database<Delete<By<Realty, realty::CreationDateTime>>> for Postgres<C>
where
    C: Connection,
{
    type Ok = ();
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Delete(by): Delete<By<Realty, realty::CreationDateTime>>,
    ) -> Result<Self::Ok, Self::Err> {
        let deadline: realty::CreationDateTime = by.into_inner();

        const SQL: &str = "\
            DELETE FROM realties \
            WHERE (SELECT COUNT(*) \
                   FROM contracts \
                   WHERE realty_id = realties.id \
                     AND terminated_at IS NULL \
                     AND (expires_at IS NULL \
                          OR expires_at > NOW())) = 0 \
              AND created_at < $1";
        self.exec(SQL, &[&deadline])
            .await
            .map_err(tracerr::wrap!())
            .map(drop)
    }
}
