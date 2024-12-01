//! [`Query`] collection related to the multiple [`Realty`].

use common::operations::By;

use crate::read;
#[cfg(doc)]
use crate::{domain::Realty, Query};

use super::DatabaseQuery;

/// Queries a list of [`Realty`].
pub type List =
    DatabaseQuery<By<read::realty::list::Page, read::realty::list::Selector>>;

/// Queries total count of [`Realty`] list items.
pub type TotalCount = DatabaseQuery<By<read::realty::list::TotalCount, ()>>;
