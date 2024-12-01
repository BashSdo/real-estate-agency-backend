//! [`Query`] collection related to the multiple [`User`]s.

use std::collections::HashMap;

use common::operations::By;

#[cfg(doc)]
use crate::Query;
use crate::{
    domain::{user, User},
    read,
};

use super::DatabaseQuery;

/// Queries multiple [`User`]s by their [`user::Id`]s.
pub type ByIds = DatabaseQuery<By<HashMap<user::Id, User>, Vec<user::Id>>>;

/// Queries a list of [`User`]s.
pub type List =
    DatabaseQuery<By<read::user::list::Page, read::user::list::Selector>>;

/// Queries total count of [`User`]s.
pub type TotalCount = DatabaseQuery<By<read::user::list::TotalCount, ()>>;
