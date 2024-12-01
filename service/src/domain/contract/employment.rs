//! Contract of an employment.

use common::{DateTime, Money};

use crate::domain::user;
#[cfg(doc)]
use crate::domain::{Contract, User};

use super::{
    CreationDateTime, Description, ExpirationDateTime, Id, Name,
    TerminationDateTime,
};

/// Employment [`Contract`].
#[derive(Clone, Debug)]
pub struct Employment {
    /// ID of this [`Contract`].
    pub id: Id,

    /// [`Name`] of this [`Contract`].
    pub name: Name,

    /// [`Description`] of this [`Contract`].
    pub description: Description,

    /// ID of the employed [`User`].
    pub employer_id: user::Id,

    /// Base salary of the employed [`User`].
    pub base_salary: Money,

    /// [`DateTime`] when this [`Contract`] was created.
    pub created_at: CreationDateTime,

    /// [`DateTime`] when this [`Contract`] expires.
    ///
    /// [`None`] means that this [`Contract`] is valid indefinitely.
    pub expires_at: Option<ExpirationDateTime>,

    /// [`DateTime`] when this [`Contract`] was terminated, if it was.
    pub terminated_at: Option<TerminationDateTime>,
}

impl Employment {
    /// Returns whether this [`Contract`] is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.terminated_at.is_none()
            && self
                .expires_at
                .map_or(true, |e| DateTime::now() < e.coerce())
    }
}
