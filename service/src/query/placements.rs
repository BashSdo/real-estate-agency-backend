//! [`Query`] collection related to the multiple [`Placement`]s.

use common::operations::By;

use crate::read::placement;
#[cfg(doc)]
use crate::{read::Placement, Query};

use super::DatabaseQuery;

/// Queries a list of [`Placement`]s.
pub type List =
    DatabaseQuery<By<placement::list::Page, placement::list::Selector>>;

/// Queries total count of [`Placement`]s.
pub type TotalCount = DatabaseQuery<By<placement::list::TotalCount, ()>>;
