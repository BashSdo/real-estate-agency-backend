//! [`Salary`] definition.

use common::{
    operations::{By, Select},
    DateTime, Money,
};
use rust_decimal::Decimal;
use std::{collections::HashMap, ops::RangeInclusive};
use tracerr::Traced;

#[cfg(doc)]
use crate::domain::{Contract, User};
use crate::{
    domain::{contract, user},
    infra::{database, Database},
    read::{self, contract::Active},
    Query, Service,
};

/// [`Query`] to calculate salaries for a given period.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Salary {
    /// Start of the period.
    pub start: DateTime,

    /// End of the period.
    pub end: DateTime,
}

/// Output of the [`Salary`] [`Query`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Output {
    /// Total count of [`Contract`]s in the period.
    pub total_contracts: read::contract::list::TotalCount,

    /// Rows of the report.
    pub rows: Vec<Row>,
}

/// Row in the [`Output`] of the [`Salary`] [`Query`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Row {
    /// ID of the [`User`] the salary is calculated for.
    pub user_id: user::Id,

    /// Number of [`Contract`]s the [`User`] made in the period.
    pub contracts: read::contract::list::TotalCount,

    /// Calculated salary for the [`User`].
    pub salary: Money,
}

impl<Db> Query<Salary> for Service<Db>
where
    Db: Database<
            Select<
                By<
                    read::contract::list::TotalCount,
                    RangeInclusive<contract::CreationDateTime>,
                >,
            >,
            Ok = read::contract::list::TotalCount,
            Err = Traced<database::Error>,
        > + Database<
            Select<
                By<
                    HashMap<user::Id, read::contract::list::TotalCount>,
                    RangeInclusive<contract::CreationDateTime>,
                >,
            >,
            Ok = HashMap<user::Id, read::contract::list::TotalCount>,
            Err = Traced<database::Error>,
        > + Database<
            Select<
                By<
                    HashMap<user::Id, Active<contract::Employment>>,
                    Vec<user::Id>,
                >,
            >,
            Ok = HashMap<user::Id, Active<contract::Employment>>,
            Err = Traced<database::Error>,
        >,
{
    type Ok = Output;
    type Err = Traced<database::Error>;

    async fn execute(
        &self,
        Salary { start, end }: Salary,
    ) -> Result<Self::Ok, Self::Err> {
        let range = RangeInclusive::new(start.coerce(), end.coerce());

        let total_count = self
            .database()
            .execute(Select(By::<read::contract::list::TotalCount, _>::new(
                range.clone(),
            )))
            .await
            .map_err(tracerr::wrap!())?;
        if i32::from(total_count) == 0 {
            return Ok(Output {
                total_contracts: total_count,
                rows: vec![],
            });
        }

        let total_by_user = self
            .database()
            .execute(Select(By::<
                HashMap<user::Id, read::contract::list::TotalCount>,
                _,
            >::new(range)))
            .await
            .map_err(tracerr::wrap!())?;

        let user_ids = total_by_user.keys().copied().collect::<Vec<_>>();
        let contracts = self
            .database()
            .execute(Select(By::<
                HashMap<user::Id, Active<contract::Employment>>,
                _,
            >::new(user_ids)))
            .await
            .map_err(tracerr::wrap!())?;

        let rows = total_by_user
            .into_iter()
            .filter_map(|(user_id, count)| {
                let Active(employment) = contracts.get(&user_id)?;

                let coef = Decimal::try_from(count / total_count)
                    .expect("in `[0..1]` range");
                let bonus = employment.base_salary.amount * coef;
                let amount = employment.base_salary.amount + bonus;

                Some(Row {
                    user_id,
                    contracts: count,
                    salary: Money {
                        amount,
                        currency: employment.base_salary.currency,
                    },
                })
            })
            .collect();

        Ok(Output {
            total_contracts: total_count,
            rows,
        })
    }
}
