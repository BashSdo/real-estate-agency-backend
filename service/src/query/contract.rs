//! [`Query`] collection related to a single [`Contract`].

use common::operations::By;

use crate::{
    domain::{contract, realty, user, Contract},
    read::contract::Active,
};
#[cfg(doc)]
use crate::{
    domain::{Realty, User},
    Query,
};

use super::DatabaseQuery;

/// Queries a [`Contract`] by its [`contract::Id`].
pub type ById = DatabaseQuery<By<Option<Contract>, contract::Id>>;

/// Queries an active [`contract::Employment`] by ID of the employed [`User`].
pub type Employment =
    DatabaseQuery<By<Option<Active<contract::Employment>>, user::Id>>;

/// Queries an active [`contract::ManagementForRent`] by ID of the related
/// [`Realty`].
pub type ManagementForRent =
    DatabaseQuery<By<Option<Active<contract::ManagementForRent>>, realty::Id>>;

/// Queries an active [`contract::ManagementForSale`] by ID of the related
/// [`Realty`].
pub type ManagementForSale =
    DatabaseQuery<By<Option<Active<contract::ManagementForSale>>, realty::Id>>;
