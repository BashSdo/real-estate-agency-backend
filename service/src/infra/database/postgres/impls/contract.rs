//! [`Contract`]-related [`Database`] implementations.

use std::{collections::HashMap, ops::RangeInclusive};

use common::{
    money,
    operations::{By, Insert, Lock, Select, Update},
    Money, Percent,
};
use itertools::Itertools as _;
use postgres_types::ToSql;
use rust_decimal::Decimal;
use tracerr::Traced;

use crate::{
    domain::{contract, realty, user, Contract},
    infra::{
        database::{
            self,
            postgres::{Connection, FuzzPattern},
            Postgres,
        },
        Database,
    },
    read::{self, contract::Active},
};

impl<C, IDs> Database<Select<By<HashMap<contract::Id, Contract>, IDs>>>
    for Postgres<C>
where
    C: Connection,
    IDs: AsRef<[contract::Id]>,
{
    type Ok = HashMap<contract::Id, Contract>;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<By<HashMap<contract::Id, Contract>, IDs>>,
    ) -> Result<Self::Ok, Self::Err> {
        let ids = by.into_inner();
        // Avoid subtle change for SQL.
        let ids: &[contract::Id] = ids.as_ref();
        if ids.is_empty() {
            return Ok(HashMap::new());
        }
        let limit = i32::try_from(ids.len()).unwrap();

        #[expect(clippy::items_after_statements, reason = "more readable")]
        const SQL: &str = "\
            SELECT id, kind, \
                   name, description, \
                   realty_id, employer_id, landlord_id, purchaser_id, \
                   price, price_currency, \
                   deposit, deposit_currency, \
                   one_time_fee, one_time_fee_currency, \
                   monthly_fee, monthly_fee_currency, \
                   percent_fee, \
                   is_placed, \
                   created_at, expires_at, terminated_at \
            FROM contracts \
            WHERE id IN (SELECT unnest($1::UUID[]) LIMIT $2::INT4) \
            LIMIT $2::INT4";
        Ok(self
            .query(SQL, &[&ids, &limit])
            .await
            .map_err(tracerr::wrap!())?
            .into_iter()
            .map(|row| {
                let id = row.get("id");
                let name = row.get("name");
                let description = row.get("description");
                let employer_id = row.get("employer_id");
                let created_at = row.get("created_at");
                let expires_at = row.get("expires_at");
                let terminated_at = row.get("terminated_at");
                let user = match row.get("kind") {
                    contract::Kind::Rent => contract::Rent {
                        id,
                        name,
                        description,
                        realty_id: row.get("realty_id"),
                        purchaser_id: row.get("purchaser_id"),
                        landlord_id: row.get("landlord_id"),
                        employer_id,
                        price: Money {
                            amount: row.get("price"),
                            currency: row.get("price_currency"),
                        },
                        deposit: row.get::<_, Option<_>>("deposit").map(
                            |amount| Money {
                                amount,
                                currency: row.get("deposit_currency"),
                            },
                        ),
                        created_at,
                        expires_at,
                        terminated_at,
                    }
                    .into(),
                    contract::Kind::Sale => contract::Sale {
                        id,
                        name,
                        description,
                        realty_id: row.get("realty_id"),
                        purchaser_id: row.get("purchaser_id"),
                        landlord_id: row.get("landlord_id"),
                        employer_id,
                        price: Money {
                            amount: row.get("price"),
                            currency: row.get("price_currency"),
                        },
                        deposit: row.get::<_, Option<_>>("deposit").map(
                            |amount| Money {
                                amount,
                                currency: row.get("deposit_currency"),
                            },
                        ),
                        created_at,
                        expires_at,
                        terminated_at,
                    }
                    .into(),
                    contract::Kind::ManagementForRent => {
                        contract::ManagementForRent {
                            id,
                            name,
                            description,
                            realty_id: row.get("realty_id"),
                            landlord_id: row.get("landlord_id"),
                            employer_id,
                            expected_price: Money {
                                amount: row.get("price"),
                                currency: row.get("price_currency"),
                            },
                            expected_deposit: row
                                .get::<_, Option<_>>("deposit")
                                .map(|amount| Money {
                                    amount,
                                    currency: row.get("deposit_currency"),
                                }),
                            one_time_fee: row
                                .get::<_, Option<_>>("one_time_fee")
                                .map(|amount| Money {
                                    amount,
                                    currency: row.get("one_time_fee_currency"),
                                }),
                            monthly_fee: row
                                .get::<_, Option<_>>("monthly_fee")
                                .map(|amount| Money {
                                    amount,
                                    currency: row.get("monthly_fee_currency"),
                                }),
                            percent_fee: row.get("percent_fee"),
                            is_placed: row.get("is_placed"),
                            created_at,
                            expires_at,
                            terminated_at,
                        }
                        .into()
                    }
                    contract::Kind::ManagementForSale => {
                        contract::ManagementForSale {
                            id,
                            name,
                            description,
                            realty_id: row.get("realty_id"),
                            landlord_id: row.get("landlord_id"),
                            employer_id,
                            expected_price: Money {
                                amount: row.get("price"),
                                currency: row.get("price_currency"),
                            },
                            expected_deposit: row
                                .get::<_, Option<_>>("deposit")
                                .map(|amount| Money {
                                    amount,
                                    currency: row.get("deposit_currency"),
                                }),
                            one_time_fee: row
                                .get::<_, Option<_>>("one_time_fee")
                                .map(|amount| Money {
                                    amount,
                                    currency: row.get("one_time_fee_currency"),
                                }),
                            monthly_fee: row
                                .get::<_, Option<_>>("monthly_fee")
                                .map(|amount| Money {
                                    amount,
                                    currency: row.get("monthly_fee_currency"),
                                }),
                            percent_fee: row.get("percent_fee"),
                            is_placed: row.get("is_placed"),
                            created_at,
                            expires_at,
                            terminated_at,
                        }
                        .into()
                    }
                    contract::Kind::Employment => contract::Employment {
                        id,
                        name,
                        description,
                        employer_id,
                        base_salary: Money {
                            amount: row.get("price"),
                            currency: row.get("price_currency"),
                        },
                        created_at,
                        expires_at,
                        terminated_at,
                    }
                    .into(),
                };
                (id, user)
            })
            .collect())
    }
}

impl<C> Database<Select<By<Option<Contract>, contract::Id>>> for Postgres<C>
where
    C: Connection,
    Self: Database<
        Select<By<HashMap<contract::Id, Contract>, [contract::Id; 1]>>,
        Ok = HashMap<contract::Id, Contract>,
        Err = Traced<database::Error>,
    >,
{
    type Ok = Option<Contract>;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<By<Option<Contract>, contract::Id>>,
    ) -> Result<Self::Ok, Self::Err> {
        let id = by.into_inner();
        Ok(self
            .execute(Select(By::new([id])))
            .await
            .map_err(tracerr::wrap!())?
            .remove(&id))
    }
}

impl<C> Database<Insert<Contract>> for Postgres<C>
where
    C: Connection,
    Self: Database<Update<Contract>, Ok = (), Err = Traced<database::Error>>,
{
    type Ok = ();
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Insert(contract): Insert<Contract>,
    ) -> Result<Self::Ok, Self::Err> {
        self.execute(Update(contract))
            .await
            .map_err(tracerr::wrap!())
    }
}

impl<C> Database<Update<Contract>> for Postgres<C>
where
    C: Connection,
{
    type Ok = ();
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Update(contract): Update<Contract>,
    ) -> Result<Self::Ok, Self::Err> {
        // Avoid subtle change for SQL.
        #[expect(clippy::type_complexity, reason = "still readable")]
        let (
            id,
            kind,
            name,
            description,
            realty_id,
            employer_id,
            landlord_id,
            purchaser_id,
            price,
            price_currency,
            deposit,
            deposit_currency,
            one_time_fee,
            one_time_fee_currency,
            monthly_fee,
            monthly_fee_currency,
            percent_fee,
            is_placed,
            created_at,
            expires_at,
            terminated_at,
        ): (
            contract::Id,
            contract::Kind,
            contract::Name,
            contract::Description,
            Option<realty::Id>,
            user::Id,
            Option<user::Id>,
            Option<user::Id>,
            Decimal,
            money::Currency,
            Option<Decimal>,
            Option<money::Currency>,
            Option<Decimal>,
            Option<money::Currency>,
            Option<Decimal>,
            Option<money::Currency>,
            Option<Percent>,
            Option<bool>,
            contract::CreationDateTime,
            Option<contract::ExpirationDateTime>,
            Option<contract::TerminationDateTime>,
        ) = match contract {
            Contract::Rent(c) => (
                c.id,
                contract::Kind::Rent,
                c.name,
                c.description,
                Some(c.realty_id),
                c.employer_id,
                Some(c.landlord_id),
                Some(c.purchaser_id),
                c.price.amount,
                c.price.currency,
                c.deposit.map(|d| d.amount),
                c.deposit.map(|d| d.currency),
                None,
                None,
                None,
                None,
                None,
                None,
                c.created_at,
                c.expires_at,
                c.terminated_at,
            ),
            Contract::Sale(c) => (
                c.id,
                contract::Kind::Sale,
                c.name,
                c.description,
                Some(c.realty_id),
                c.employer_id,
                Some(c.landlord_id),
                Some(c.purchaser_id),
                c.price.amount,
                c.price.currency,
                c.deposit.map(|d| d.amount),
                c.deposit.map(|d| d.currency),
                None,
                None,
                None,
                None,
                None,
                None,
                c.created_at,
                c.expires_at,
                c.terminated_at,
            ),
            Contract::ManagementForRent(c) => (
                c.id,
                contract::Kind::ManagementForRent,
                c.name,
                c.description,
                Some(c.realty_id),
                c.employer_id,
                Some(c.landlord_id),
                None,
                c.expected_price.amount,
                c.expected_price.currency,
                c.expected_deposit.map(|d| d.amount),
                c.expected_deposit.map(|d| d.currency),
                c.one_time_fee.map(|f| f.amount),
                c.one_time_fee.map(|f| f.currency),
                c.monthly_fee.map(|f| f.amount),
                c.monthly_fee.map(|f| f.currency),
                c.percent_fee,
                Some(c.is_placed),
                c.created_at,
                c.expires_at,
                c.terminated_at,
            ),
            Contract::ManagementForSale(c) => (
                c.id,
                contract::Kind::ManagementForSale,
                c.name,
                c.description,
                Some(c.realty_id),
                c.employer_id,
                Some(c.landlord_id),
                None,
                c.expected_price.amount,
                c.expected_price.currency,
                c.expected_deposit.map(|d| d.amount),
                c.expected_deposit.map(|d| d.currency),
                c.one_time_fee.map(|f| f.amount),
                c.one_time_fee.map(|f| f.currency),
                c.monthly_fee.map(|f| f.amount),
                c.monthly_fee.map(|f| f.currency),
                c.percent_fee,
                Some(c.is_placed),
                c.created_at,
                c.expires_at,
                c.terminated_at,
            ),
            Contract::Employment(c) => (
                c.id,
                contract::Kind::Employment,
                c.name,
                c.description,
                None,
                c.employer_id,
                None,
                None,
                c.base_salary.amount,
                c.base_salary.currency,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                c.created_at,
                c.expires_at,
                c.terminated_at,
            ),
        };

        const SQL: &str = "\
            INSERT INTO contracts (\
                id, kind, \
                name, description, \
                realty_id, employer_id, landlord_id, purchaser_id, \
                price, price_currency, \
                deposit, deposit_currency, \
                one_time_fee, one_time_fee_currency, \
                monthly_fee, monthly_fee_currency, \
                percent_fee, \
                is_placed, \
                created_at, expires_at, terminated_at\
            ) VALUES (\
                $1::UUID, $2::INT2, \
                $3::VARCHAR, $4::VARCHAR, \
                $5::UUID, $6::UUID, $7::UUID, $8::UUID, \
                $9::NUMERIC, $10::INT2, \
                $11::NUMERIC, $12::INT2, \
                $13::NUMERIC, $14::INT2, \
                $15::NUMERIC, $16::INT2, \
                $17::NUMERIC, \
                $18::BOOLEAN, \
                $19::TIMESTAMPTZ, $20::TIMESTAMPTZ, $21::TIMESTAMPTZ\
            ) \
            ON CONFLICT (id) DO UPDATE \
            SET kind = EXCLUDED.kind, \
                name = EXCLUDED.name, \
                description = EXCLUDED.description, \
                realty_id = EXCLUDED.realty_id, \
                employer_id = EXCLUDED.employer_id, \
                landlord_id = EXCLUDED.landlord_id, \
                purchaser_id = EXCLUDED.purchaser_id, \
                price = EXCLUDED.price, \
                price_currency = EXCLUDED.price_currency, \
                deposit = EXCLUDED.deposit, \
                deposit_currency = EXCLUDED.deposit_currency, \
                one_time_fee = EXCLUDED.one_time_fee, \
                one_time_fee_currency = EXCLUDED.one_time_fee_currency, \
                monthly_fee = EXCLUDED.monthly_fee, \
                monthly_fee_currency = EXCLUDED.monthly_fee_currency, \
                percent_fee = EXCLUDED.percent_fee, \
                is_placed = EXCLUDED.is_placed, \
                created_at = EXCLUDED.created_at, \
                expires_at = EXCLUDED.expires_at, \
                terminated_at = EXCLUDED.terminated_at";
        self.exec(
            SQL,
            &[
                &id,
                &kind,
                &name,
                &description,
                &realty_id,
                &employer_id,
                &landlord_id,
                &purchaser_id,
                &price,
                &price_currency,
                &deposit,
                &deposit_currency,
                &one_time_fee,
                &one_time_fee_currency,
                &monthly_fee,
                &monthly_fee_currency,
                &percent_fee,
                &is_placed,
                &created_at,
                &expires_at,
                &terminated_at,
            ],
        )
        .await
        .map_err(tracerr::wrap!())
        .map(drop)
    }
}

impl<C, IDs>
    Database<Select<By<HashMap<user::Id, Active<contract::Employment>>, IDs>>>
    for Postgres<C>
where
    C: Connection,
    IDs: AsRef<[user::Id]>,
    Self: Database<
        Select<By<HashMap<contract::Id, Contract>, Vec<contract::Id>>>,
        Ok = HashMap<contract::Id, Contract>,
        Err = Traced<database::Error>,
    >,
{
    type Ok = HashMap<user::Id, Active<contract::Employment>>;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<
            By<HashMap<user::Id, Active<contract::Employment>>, IDs>,
        >,
    ) -> Result<Self::Ok, Self::Err> {
        let user_ids = by.into_inner();
        // Avoid subtle change for SQL.
        let user_ids: &[user::Id] = user_ids.as_ref();
        let limit = i32::try_from(user_ids.len()).unwrap();

        const SQL: &str = "\
            SELECT id \
            FROM contracts \
            WHERE kind = $1::INT2 \
              AND employer_id IN (SELECT unnest($2::UUID[]) LIMIT $3::INT4) \
              AND terminated_at IS NULL \
              AND (expires_at IS NULL OR expires_at > NOW()) \
            LIMIT $3::INT4";
        let contract_ids = self
            .query(SQL, &[&contract::Kind::Employment, &user_ids, &limit])
            .await
            .map_err(tracerr::wrap!())
            .map(|rows| {
                rows.into_iter()
                    .map(|row| row.get("id"))
                    .collect::<Vec<_>>()
            })?;

        self.execute(Select(By::<HashMap<contract::Id, Contract>, _>::new(
            contract_ids,
        )))
        .await
        .map_err(tracerr::wrap!())
        .map(|c| {
            c.into_values()
                .filter(Contract::is_active)
                .map(|c| match c {
                    Contract::Employment(c) => (c.employer_id, Active(c)),
                    Contract::ManagementForRent(_)
                    | Contract::ManagementForSale(_)
                    | Contract::Rent(_)
                    | Contract::Sale(_) => unreachable!("already checked"),
                })
                .collect()
        })
    }
}

impl<C> Database<Select<By<Option<Active<contract::Employment>>, user::Id>>>
    for Postgres<C>
where
    C: Connection,
    Self: Database<
        Select<
            By<HashMap<user::Id, Active<contract::Employment>>, [user::Id; 1]>,
        >,
        Ok = HashMap<user::Id, Active<contract::Employment>>,
        Err = Traced<database::Error>,
    >,
{
    type Ok = Option<Active<contract::Employment>>;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<By<Option<Active<contract::Employment>>, user::Id>>,
    ) -> Result<Self::Ok, Self::Err> {
        let user_id = by.into_inner();
        self.execute(Select(By::new([user_id])))
            .await
            .map_err(tracerr::wrap!())
            .map(|mut c| c.remove(&user_id))
    }
}

impl<C>
    Database<
        Select<By<Option<Active<contract::ManagementForRent>>, realty::Id>>,
    > for Postgres<C>
where
    C: Connection,
    Self: Database<
        Select<By<Option<Contract>, contract::Id>>,
        Ok = Option<Contract>,
        Err = Traced<database::Error>,
    >,
{
    type Ok = Option<Active<contract::ManagementForRent>>;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<
            By<Option<Active<contract::ManagementForRent>>, realty::Id>,
        >,
    ) -> Result<Self::Ok, Self::Err> {
        // Avoid subtle change for SQL.
        let realty_id: realty::Id = by.into_inner();

        const SQL: &str = "\
            SELECT id \
            FROM contracts \
            WHERE kind = $1::INT2 \
              AND realty_id = $2::UUID \
              AND terminated_at IS NULL \
              AND (expires_at IS NULL OR expires_at > NOW()) \
            LIMIT 1";
        let Some(row) = self
            .query_opt(SQL, &[&contract::Kind::ManagementForRent, &realty_id])
            .await
            .map_err(tracerr::wrap!())?
        else {
            return Ok(None);
        };

        self.execute(Select(By::new(row.get("id"))))
            .await
            .map_err(tracerr::wrap!())
            .map(|c| {
                c.map(|c| match c {
                    Contract::ManagementForRent(c) => Active(c),
                    Contract::Employment(_)
                    | Contract::ManagementForSale(_)
                    | Contract::Rent(_)
                    | Contract::Sale(_) => unreachable!("already checked"),
                })
            })
    }
}

impl<C>
    Database<
        Select<By<Option<Active<contract::ManagementForSale>>, realty::Id>>,
    > for Postgres<C>
where
    C: Connection,
    Self: Database<
        Select<By<Option<Contract>, contract::Id>>,
        Ok = Option<Contract>,
        Err = Traced<database::Error>,
    >,
{
    type Ok = Option<Active<contract::ManagementForSale>>;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<
            By<Option<Active<contract::ManagementForSale>>, realty::Id>,
        >,
    ) -> Result<Self::Ok, Self::Err> {
        // Avoid subtle change for SQL.
        let realty_id: realty::Id = by.into_inner();

        const SQL: &str = "\
            SELECT id \
            FROM contracts \
            WHERE kind = $1::INT2 \
              AND realty_id = $2::UUID \
              AND terminated_at IS NULL \
              AND (expires_at IS NULL OR expires_at > NOW()) \
            LIMIT 1";
        let Some(row) = self
            .query_opt(SQL, &[&contract::Kind::ManagementForSale, &realty_id])
            .await
            .map_err(tracerr::wrap!())?
        else {
            return Ok(None);
        };

        self.execute(Select(By::new(row.get("id"))))
            .await
            .map_err(tracerr::wrap!())
            .map(|c| {
                c.map(|c| match c {
                    Contract::ManagementForSale(c) => Active(c),
                    Contract::Employment(_)
                    | Contract::ManagementForRent(_)
                    | Contract::Rent(_)
                    | Contract::Sale(_) => unreachable!("already checked"),
                })
            })
    }
}

impl<C> Database<Lock<By<Contract, contract::Id>>> for Postgres<C>
where
    C: Connection,
{
    type Ok = ();
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Lock(by): Lock<By<Contract, contract::Id>>,
    ) -> Result<Self::Ok, Self::Err> {
        // Avoid subtle change for SQL.
        let id: contract::Id = by.into_inner();

        const SQL: &str = "\
            INSERT INTO contracts_lock \
            VALUES ($1::UUID) \
            ON CONFLICT (id) DO NOTHING";
        self.query(SQL, &[&id])
            .await
            .map_err(tracerr::wrap!())
            .map(drop)
    }
}

impl<C>
    Database<
        Select<By<read::contract::list::Page, read::contract::list::Selector>>,
    > for Postgres<C>
where
    C: Connection,
{
    type Ok = read::contract::list::Page;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<
            By<read::contract::list::Page, read::contract::list::Selector>,
        >,
    ) -> Result<Self::Ok, Self::Err> {
        let read::contract::list::Selector {
            arguments,
            filter: read::contract::list::Filter { name },
        } = by.into_inner();

        let limit = i32::try_from(arguments.limit()).unwrap() + 1;

        let mut ps: Vec<&(dyn ToSql + Sync)> = vec![&limit];

        let cursor_idx = arguments.cursor().map(|c| {
            ps.push(c);
            ps.len()
        });
        let name_idx = name.as_ref().map(|n| {
            ps.push(n);
            ps.len()
        });

        let name_pattern = name.as_ref().map(|n| FuzzPattern::new(n.as_ref()));
        let name_pattern_idx = name_pattern.as_ref().map(|n| {
            ps.push(n);
            ps.len()
        });

        let sql = format!(
            "SELECT id, kind \
             FROM contracts \
             WHERE true \
                   {cursor} \
                   {name_filtering} \
             ORDER BY {name_ordering} \
                      id ASC \
             LIMIT $1::INT4",
            cursor = cursor_idx.into_iter().format_with("", |idx, f| {
                let op = arguments.kind().operator();
                f(&format_args!("AND id {op} ${idx}::UUID"))
            }),
            name_filtering =
                name_pattern_idx.into_iter().format_with("", |idx, f| {
                    f(&format_args!(
                        "AND LOWER(name) SIMILAR TO LOWER(${idx}::VARCHAR)"
                    ))
                }),
            name_ordering = name_idx.into_iter().format_with("", |idx, f| {
                let order = arguments.kind().order().sql();
                f(&format_args!(
                    "LEVENSHTEIN(name, ${idx}::VARCHAR, 1, 1, 0) {order},"
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
                let kind = row.get("kind");
                (id, (id, kind))
            })
            .collect::<Vec<_>>();

        Ok(read::contract::list::Page::new(&arguments, edges, has_more))
    }
}

impl<C> Database<Select<By<read::contract::list::TotalCount, ()>>>
    for Postgres<C>
where
    C: Connection,
{
    type Ok = read::contract::list::TotalCount;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(_): Select<By<read::contract::list::TotalCount, ()>>,
    ) -> Result<Self::Ok, Self::Err> {
        const SQL: &str = "\
            SELECT COUNT(*)::INT4 \
            FROM contracts";
        self.query_opt(SQL, &[])
            .await
            .map_err(tracerr::wrap!())
            .map(|row| row.expect("always exists").get::<_, i32>(0).into())
    }
}

impl<C>
    Database<
        Select<
            By<
                read::contract::list::TotalCount,
                RangeInclusive<contract::CreationDateTime>,
            >,
        >,
    > for Postgres<C>
where
    C: Connection,
{
    type Ok = read::contract::list::TotalCount;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<
            By<
                read::contract::list::TotalCount,
                RangeInclusive<contract::CreationDateTime>,
            >,
        >,
    ) -> Result<Self::Ok, Self::Err> {
        // Avoid subtle change for SQL.
        let range: RangeInclusive<contract::CreationDateTime> = by.into_inner();

        const SQL: &str = "\
            SELECT COUNT(id)::INT4 \
            FROM contracts \
            WHERE kind IN (SELECT unnest($1::INT2[]) LIMIT $2::INT4) \
              AND created_at >= $3::TIMESTAMPTZ \
              AND created_at <= $4::TIMESTAMPTZ";
        self.query_opt(
            SQL,
            &[
                &[
                    contract::Kind::ManagementForRent,
                    contract::Kind::ManagementForSale,
                    contract::Kind::Rent,
                    contract::Kind::Sale,
                ]
                .as_slice(),
                &4i32,
                range.start(),
                range.end(),
            ],
        )
        .await
        .map_err(tracerr::wrap!())
        .map(|row| row.expect("always exists").get::<_, i32>(0).into())
    }
}

impl<C>
    Database<
        Select<
            By<
                HashMap<user::Id, read::contract::list::TotalCount>,
                RangeInclusive<contract::CreationDateTime>,
            >,
        >,
    > for Postgres<C>
where
    C: Connection,
{
    type Ok = HashMap<user::Id, read::contract::list::TotalCount>;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<
            By<
                HashMap<user::Id, read::contract::list::TotalCount>,
                RangeInclusive<contract::CreationDateTime>,
            >,
        >,
    ) -> Result<Self::Ok, Self::Err> {
        // Avoid subtle change for SQL.
        let range: RangeInclusive<contract::CreationDateTime> = by.into_inner();

        const SQL: &str = "\
            SELECT employer_id, COUNT(id)::INT4 AS count \
            FROM contracts \
            WHERE kind IN (SELECT unnest($1::INT2[]) LIMIT $2::INT4) \
              AND created_at >= $3::TIMESTAMPTZ \
              AND created_at <= $4::TIMESTAMPTZ \
            GROUP BY employer_id";
        self.query(
            SQL,
            &[
                &[
                    contract::Kind::ManagementForRent,
                    contract::Kind::ManagementForSale,
                    contract::Kind::Rent,
                    contract::Kind::Sale,
                ]
                .as_slice(),
                &4i32,
                range.start(),
                range.end(),
            ],
        )
        .await
        .map_err(tracerr::wrap!())
        .map(|rows| {
            rows.into_iter()
                .map(|row| {
                    let employer_id = row.get::<_, user::Id>("employer_id");
                    let count = row.get::<_, i32>("count");
                    (employer_id, count.into())
                })
                .collect()
        })
    }
}
