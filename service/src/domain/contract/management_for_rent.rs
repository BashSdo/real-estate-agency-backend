//! [`ManagementForRent`] [`Contract`] definition.

use common::{DateTime, Money, Percent};

use crate::domain::{realty, user};

use super::{
    CreationDateTime, Description, ExpirationDateTime, Id, Name,
    TerminationDateTime,
};
#[cfg(doc)]
use crate::domain::{Contract, Realty, User};

/// A [`Contract`] that allows platform to manage a [`Realty`] for a rent.
#[derive(Clone, Debug)]
pub struct ManagementForRent {
    /// ID of this [`Contract`].
    pub id: Id,

    /// [`Name`] of this [`Contract`].
    pub name: Name,

    /// [`Description`] of this [`Contract`].
    pub description: Description,

    /// ID of the [`Realty`] this [`Contract`] allows to manage.
    pub realty_id: realty::Id,

    /// ID of the [`User`] who owns the [`Realty`].
    pub landlord_id: user::Id,

    /// ID of the [`User`] who manages the [`Realty`].
    pub employer_id: user::Id,

    /// Expected rent price for the [`Realty`].
    pub expected_price: Money,

    /// Expected deposit for the [`Realty`].
    pub expected_deposit: Option<Money>,

    /// One-time fee for the management taken at the moment of this
    /// [`Contract`] signing.
    pub one_time_fee: Option<Money>,

    /// Monthly fee for the management taken every month for
    /// the duration of this [`Contract`].
    pub monthly_fee: Option<Money>,

    /// Percent fee for the management taken from the rent or sale price.
    pub percent_fee: Option<Percent>,

    /// Indicator whether [`Realty`] of this [`Contract`] is placed
    /// for rent.
    pub is_placed: bool,

    /// [`DateTime`] when this [`Contract`] was created.
    pub created_at: CreationDateTime,

    /// [`DateTime`] when this [`Contract`] expires.
    ///
    /// [`None`] means that this [`Contract`] is valid indefinitely.
    pub expires_at: Option<ExpirationDateTime>,

    /// [`DateTime`] when this [`Contract`] was terminated, if it was.
    pub terminated_at: Option<TerminationDateTime>,
}

impl ManagementForRent {
    /// Returns whether this [`Contract`] is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.terminated_at.is_none()
            && self
                .expires_at
                .map_or(true, |e| DateTime::now() < e.coerce())
    }
}
