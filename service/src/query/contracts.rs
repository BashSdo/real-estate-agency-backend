//! [`Query`] collection related to the multiple [`Contract`]s.

use common::operations::By;

use crate::read;
#[cfg(doc)]
use crate::{domain::Contract, Query};

use super::DatabaseQuery;

/// Queries a list of [`Contract`]s.
pub type List = DatabaseQuery<
    By<read::contract::list::Page, read::contract::list::Selector>,
>;

/// Queries total count of [`Contract`]s.
pub type TotalCount = DatabaseQuery<By<read::contract::list::TotalCount, ()>>;
