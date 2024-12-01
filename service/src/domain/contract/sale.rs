//! [`Sale`] [`Contract`] definition.

use common::{DateTime, Money};

use crate::domain::{realty, user};
#[cfg(doc)]
use crate::domain::{Contract, Realty, User};

use super::{
    CreationDateTime, Description, ExpirationDateTime, Id, Name,
    TerminationDateTime,
};

/// [`Contract`] about a [`User`] to buy a [`Realty`].
#[derive(Clone, Debug)]
pub struct Sale {
    /// ID of this [`Contract`].
    pub id: Id,

    /// [`Name`] of this [`Contract`].
    pub name: Name,

    /// [`Description`] of this [`Contract`].
    pub description: Description,

    /// ID of the [`Realty`] this [`Contract`] is about.
    pub realty_id: realty::Id,

    /// ID of the [`User`] who bought the [`Realty`].
    pub purchaser_id: user::Id,

    /// ID of the [`User`] who sold the [`Realty`].
    pub landlord_id: user::Id,

    /// ID of the [`User`] who manages the [`Realty`].
    pub employer_id: user::Id,

    /// Price for which the [`Realty`] was sold.
    pub price: Money,

    /// Deposit was previously paid by the purchaser, if any.
    pub deposit: Option<Money>,

    /// [`DateTime`] when this [`Contract`] was created.
    pub created_at: CreationDateTime,

    /// [`DateTime`] when this [`Contract`] expires.
    ///
    /// [`None`] means that this [`Contract`] is valid indefinitely.
    pub expires_at: Option<ExpirationDateTime>,

    /// [`DateTime`] when this [`Contract`] was terminated, if it was.
    pub terminated_at: Option<TerminationDateTime>,
}

impl Sale {
    /// Returns whether this [`Contract`] is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.terminated_at.is_none()
            && self
                .expires_at
                .map_or(true, |e| DateTime::now() < e.coerce())
    }
}
