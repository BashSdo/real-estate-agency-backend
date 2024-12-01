//! [`Query`] collection related to a single [`Realty`].

use common::operations::By;

use crate::domain::{realty, Realty};
#[cfg(doc)]
use crate::Query;

use super::DatabaseQuery;

/// Queries a [`Realty`] by its [`realty::Id`].
pub type ById = DatabaseQuery<By<Option<Realty>, realty::Id>>;
