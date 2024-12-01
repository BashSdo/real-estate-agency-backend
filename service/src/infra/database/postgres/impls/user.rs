//! [`User`]-related [`Database`] implementations.

use std::collections::HashMap;

use common::operations::{By, Insert, Lock, Select, Update};
use itertools::Itertools as _;
use postgres_types::ToSql;
use tracerr::Traced;

use crate::{
    domain::{user, User},
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

impl<C, IDs> Database<Select<By<HashMap<user::Id, User>, IDs>>> for Postgres<C>
where
    C: Connection,
    IDs: AsRef<[user::Id]>,
{
    type Ok = HashMap<user::Id, User>;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<By<HashMap<user::Id, User>, IDs>>,
    ) -> Result<Self::Ok, Self::Err> {
        let ids = by.into_inner();
        // Avoid subtle change for SQL.
        let ids: &[user::Id] = ids.as_ref();
        if ids.is_empty() {
            return Ok(HashMap::new());
        }
        let limit = i32::try_from(ids.len()).unwrap();

        const SQL: &str = "\
            SELECT id, name, \
                   login, password_hash, \
                   email, phone, \
                   created_at, deleted_at \
            FROM users \
            WHERE id IN (SELECT unnest($1::UUID[]) LIMIT $2::INT4) \
                  AND deleted_at IS NULL \
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
                    User {
                        id,
                        name: row.get("name"),
                        login: row.get("login"),
                        password_hash: row.get("password_hash"),
                        email: row.get("email"),
                        phone: row.get("phone"),
                        created_at: row.get("created_at"),
                        deleted_at: row.get("deleted_at"),
                    },
                )
            })
            .collect())
    }
}

impl<C> Database<Select<By<Option<User>, user::Id>>> for Postgres<C>
where
    C: Connection,
    Self: Database<
        Select<By<HashMap<user::Id, User>, [user::Id; 1]>>,
        Ok = HashMap<user::Id, User>,
        Err = Traced<database::Error>,
    >,
{
    type Ok = Option<User>;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<By<Option<User>, user::Id>>,
    ) -> Result<Self::Ok, Self::Err> {
        let id = by.into_inner();
        Ok(self
            .execute(Select(By::new([id])))
            .await
            .map_err(tracerr::wrap!())?
            .remove(&id))
    }
}

impl<C> Database<Insert<User>> for Postgres<C>
where
    C: Connection,
    Self: Database<Update<User>, Ok = (), Err = Traced<database::Error>>,
{
    type Ok = ();
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Insert(user): Insert<User>,
    ) -> Result<Self::Ok, Self::Err> {
        self.execute(Update(user)).await.map_err(tracerr::wrap!())
    }
}

impl<C> Database<Update<User>> for Postgres<C>
where
    C: Connection,
{
    type Ok = ();
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Update(user): Update<User>,
    ) -> Result<Self::Ok, Self::Err> {
        let User {
            id,
            name,
            login,
            password_hash,
            email,
            phone,
            created_at,
            deleted_at,
        } = user;

        const SQL: &str = "\
            INSERT INTO users (\
                id, name, \
                login, password_hash, \
                email, phone, \
                created_at, deleted_at\
            ) \
            VALUES (\
                $1::UUID, \
                $2::VARCHAR, \
                $3::VARCHAR, $4::VARCHAR, \
                $5::VARCHAR, $6::VARCHAR, \
                $7::TIMESTAMPTZ, $8::TIMESTAMPTZ\
            ) \
            ON CONFLICT (id) DO UPDATE \
            SET name = EXCLUDED.name, \
                login = EXCLUDED.login, \
                password_hash = EXCLUDED.password_hash, \
                email = EXCLUDED.email, \
                phone = EXCLUDED.phone, \
                created_at = EXCLUDED.created_at, \
                deleted_at = EXCLUDED.deleted_at";
        self.exec(
            SQL,
            &[
                &id,
                &name,
                &login,
                &password_hash,
                &email,
                &phone,
                &created_at,
                &deleted_at,
            ],
        )
        .await
        .map_err(tracerr::wrap!())
        .map(drop)
    }
}

impl<C> Database<Lock<By<User, user::Id>>> for Postgres<C>
where
    C: Connection,
{
    type Ok = ();
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Lock(by): Lock<By<User, user::Id>>,
    ) -> Result<Self::Ok, Self::Err> {
        // Avoid subtle change for SQL.
        let id: user::Id = by.into_inner();

        const SQL: &str = "\
            INSERT INTO users_lock \
            VALUES ($1::UUID) \
            ON CONFLICT (id) DO NOTHING";
        self.query(SQL, &[&id])
            .await
            .map_err(tracerr::wrap!())
            .map(drop)
    }
}

impl<'l, C> Database<Select<By<Option<User>, &'l user::Login>>> for Postgres<C>
where
    C: Connection,
    Self: Database<
        Select<By<Option<User>, user::Id>>,
        Ok = Option<User>,
        Err = Traced<database::Error>,
    >,
{
    type Ok = Option<User>;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<By<Option<User>, &'l user::Login>>,
    ) -> Result<Self::Ok, Self::Err> {
        let login = by.into_inner();

        const SQL: &str = "\
            SELECT id \
            FROM users \
            WHERE login = $1::VARCHAR \
              AND deleted_at IS NULL \
            LIMIT 1";
        let Some(row) = self
            .query_opt(SQL, &[&login])
            .await
            .map_err(tracerr::wrap!())?
        else {
            return Ok(None);
        };

        let user_id = row.get("id");
        self.execute(Select(By::new(user_id)))
            .await
            .map_err(tracerr::wrap!())
    }
}

impl<C> Database<Select<By<read::user::list::Page, read::user::list::Selector>>>
    for Postgres<C>
where
    C: Connection,
{
    type Ok = read::user::list::Page;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<
            By<read::user::list::Page, read::user::list::Selector>,
        >,
    ) -> Result<Self::Ok, Self::Err> {
        let read::user::list::Selector {
            arguments,
            filter: read::user::list::Filter { name },
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
            "SELECT id \
             FROM users \
             WHERE deleted_at IS NULL \
                   {cursor} \
                   {name_filtering} \
             ORDER BY {name_ordering} \
                      id {order} \
             LIMIT $1::INT4",
            cursor = cursor_idx.into_iter().format_with("", |idx, f| {
                let op = arguments.kind().operator();
                f(&format_args!("AND id {op} ${idx}::UUID"))
            }),
            order = arguments.kind().order().sql(),
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
                (id, id)
            })
            .collect::<Vec<_>>();

        Ok(read::user::list::Page::new(&arguments, edges, has_more))
    }
}

impl<C> Database<Select<By<read::user::list::TotalCount, ()>>> for Postgres<C>
where
    C: Connection,
{
    type Ok = read::user::list::TotalCount;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(_): Select<By<read::user::list::TotalCount, ()>>,
    ) -> Result<Self::Ok, Self::Err> {
        const SQL: &str = "\
            SELECT COUNT(*)::INT4 \
            FROM users \
            WHERE deleted_at IS NULL";
        self.query_opt(SQL, &[])
            .await
            .map_err(tracerr::wrap!())
            .map(|row| row.expect("always exists").get::<_, i32>(0).into())
    }
}
