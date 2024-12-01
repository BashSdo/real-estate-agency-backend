//! [`Salary`] report definition.

use std::sync::OnceLock;

use common::Money;
use derive_more::From;
use juniper::graphql_object;
use service::query;

#[cfg(doc)]
use crate::api::User;
use crate::{api, Context};

/// Report calculating salaries of [`User`]-employees.
#[derive(Clone, Debug)]
pub struct Salary {
    /// Underlying [`query::report::salary::Output`].
    output: query::report::salary::Output,

    /// [`Row`]s of this report.
    rows: OnceLock<Vec<Row>>,
}

impl From<query::report::salary::Output> for Salary {
    fn from(output: query::report::salary::Output) -> Self {
        Self {
            output,
            rows: OnceLock::new(),
        }
    }
}

/// Report calculating salaries of `User`-employees.
#[graphql_object(name = "SalaryReport", context = Context)]
impl Salary {
    /// Total number of `Contract`s created within the report period.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "SalaryReport.totalContractsCount",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    #[must_use]
    pub fn total_contracts_count(&self) -> i32 {
        self.output.total_contracts.into()
    }

    /// `SalaryReportRow`s of this report.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "SalaryReport.rows",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    #[must_use]
    pub fn rows(&self) -> &[Row] {
        self.rows
            .get_or_init(|| {
                self.output.rows.iter().copied().map(Row::from).collect()
            })
            .as_slice()
    }
}

/// Row of a [`Salary`] report.
#[derive(Clone, Debug)]
pub struct Row {
    /// Underlying [`query::report::salary::Row`].
    row: query::report::salary::Row,

    /// [`User`] this [`Row`] is about.
    user: api::User,
}

impl From<query::report::salary::Row> for Row {
    fn from(row: query::report::salary::Row) -> Self {
        Self {
            // SAFETY: `Row` is constructed from a valid `user_id`.
            #[expect(
                clippy::allow_attributes,
                reason = "TODO: Remove once clippy is fixed"
            )]
            #[allow(unsafe_code, reason = "invariants are preserved")]
            user: unsafe { api::User::new_unchecked(row.user_id) },
            row,
        }
    }
}

/// Row of a `SalaryReport`.
#[graphql_object(name = "SalaryReportRow", context = Context)]
impl Row {
    /// `User` this `Row` is about.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "SalaryReportRow.user",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    #[must_use]
    pub fn user(&self) -> &api::User {
        &self.user
    }

    /// Number of `Contract`s signed by the `User` within the report period.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "SalaryReportRow.contractsCount",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    #[must_use]
    pub fn contracts_count(&self) -> i32 {
        self.row.contracts.into()
    }

    /// Total salary of the `User` within the report period.
    #[tracing::instrument(
        skip_all,
        fields(
            gql.name = "SalaryReportRow.salary",
            otel.name = api::Query::SPAN_NAME,
        ),
    )]
    #[must_use]
    pub fn salary(&self) -> Money {
        self.row.salary
    }
}
