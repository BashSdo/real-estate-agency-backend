//! [`Query`] collection related to a single [`User`].

use common::operations::By;

use crate::domain::{user, User};
#[cfg(doc)]
use crate::Query;

use super::DatabaseQuery;

/// Queries a [`User`] by its [`user::Id`].
pub type ById = DatabaseQuery<By<Option<User>, user::Id>>;
