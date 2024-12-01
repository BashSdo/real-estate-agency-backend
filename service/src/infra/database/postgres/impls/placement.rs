use common::operations::{By, Select};
use itertools::Itertools as _;
use postgres_types::ToSql;
use tracerr::Traced;

use crate::{
    domain::contract,
    infra::{
        database::{self, postgres::Connection, Postgres},
        Database,
    },
    read::{placement, Placement},
};

impl<C> Database<Select<By<placement::list::Page, placement::list::Selector>>>
    for Postgres<C>
where
    C: Connection,
{
    type Ok = placement::list::Page;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(by): Select<
            By<placement::list::Page, placement::list::Selector>,
        >,
    ) -> Result<Self::Ok, Self::Err> {
        let placement::list::Selector {
            arguments,
            filter: placement::list::Filter { rent, sale },
        } = by.into_inner();

        let limit = i32::try_from(arguments.limit()).unwrap() + 1;

        let mut ps: Vec<&(dyn ToSql + Sync)> = vec![
            &contract::Kind::ManagementForRent,
            &contract::Kind::ManagementForSale,
            &limit,
        ];

        let sql = format!(
            "SELECT realty_id, \
                    rent_contract_id, \
                    sale_contract_id \
             FROM (SELECT id AS realty_id, \
                          (SELECT id \
                           FROM contracts \
                           WHERE kind = $1::INT2 \
                             AND is_placed \
                             AND terminated_at IS NULL \
                             AND (expires_at IS NULL \
                                  OR expires_at > NOW()) \
                             AND realty_id = realties.id \
                           LIMIT 1) AS rent_contract_id, \
                          (SELECT id \
                           FROM contracts \
                           WHERE kind = $2::INT2 \
                             AND is_placed \
                             AND terminated_at IS NULL \
                             AND (expires_at IS NULL \
                                  OR expires_at > NOW()) \
                             AND realty_id = realties.id \
                           LIMIT 1) AS sale_contract_id \
                   FROM realties) AS realty \
             WHERE (rent_contract_id IS NOT NULL \
                    OR sale_contract_id IS NOT NULL) \
                   {cursor} \
                   {no_rent} \
                   {no_sale} \
             ORDER BY realty_id {order}, \
                      rent_contract_id {order}, \
                      sale_contract_id {order}
             LIMIT $3::INT4",
            cursor =
                arguments.cursor().into_iter().format_with("", |cursor, f| {
                    let op = arguments.kind().operator();

                    ps.push(cursor);
                    let idx = ps.len();

                    f(&format_args!("AND realty_id {op} ${idx}::UUID"))
                }),
            no_rent = (!rent)
                .then_some("AND rent_contract_id IS NULL")
                .unwrap_or_default(),
            no_sale = (!sale)
                .then_some("AND sale_contract_id IS NULL")
                .unwrap_or_default(),
            order = arguments.kind().order().sql(),
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
                let realty_id = row.get("realty_id");
                (
                    realty_id,
                    Placement {
                        realty_id,
                        rent_contract_id: row.get("rent_contract_id"),
                        sale_contract_id: row.get("sale_contract_id"),
                    },
                )
            })
            .collect::<Vec<_>>();

        Ok(placement::list::Page::new(&arguments, edges, has_more))
    }
}

impl<C> Database<Select<By<placement::list::TotalCount, ()>>> for Postgres<C>
where
    C: Connection,
{
    type Ok = placement::list::TotalCount;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Select(_): Select<By<placement::list::TotalCount, ()>>,
    ) -> Result<Self::Ok, Self::Err> {
        const SQL: &str = "\
            SELECT COUNT(*)::INT4 \
            FROM realties \
            WHERE EXISTS(SELECT id \
                   FROM contracts \
                   WHERE kind = $1::INT2 \
                     AND is_placed \
                     AND terminated_at IS NULL \
                     AND (expires_at IS NULL \
                          OR expires_at > NOW()) \
                     AND realty_id = realties.id \
                   LIMIT 1) \
               OR EXISTS(SELECT id \
                   FROM contracts \
                   WHERE kind = $2::INT2 \
                     AND is_placed \
                     AND terminated_at IS NULL \
                     AND (expires_at IS NULL \
                          OR expires_at > NOW()) \
                     AND realty_id = realties.id \
                   LIMIT 1)";
        self.query_opt(
            SQL,
            &[
                &contract::Kind::ManagementForRent,
                &contract::Kind::ManagementForSale,
            ],
        )
        .await
        .map_err(tracerr::wrap!())
        .map(|row| row.expect("always exists").get::<_, i32>(0).into())
    }
}
