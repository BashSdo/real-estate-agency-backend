//! Abstract operations.

use std::marker::PhantomData;

use crate::Handler;

/// Operation to insert a value.
#[derive(Clone, Copy, Debug)]
pub struct Insert<T>(pub T);

/// Operation to update a value.
#[derive(Clone, Copy, Debug)]
pub struct Update<T>(pub T);

/// Operation to delete a value.
#[derive(Clone, Copy, Debug)]
pub struct Delete<T>(pub T);

/// Operation to select a value.
#[derive(Clone, Copy, Debug)]
pub struct Select<T>(pub T);

/// Operation to lock a value.
#[derive(Clone, Copy, Debug)]
pub struct Lock<T>(pub T);

/// Operation to start a value.
#[derive(Clone, Copy, Debug)]
pub struct Start<T>(pub T);

/// Operation to perform a value.
#[derive(Clone, Copy, Debug)]
pub struct Perform<T>(pub T);

/// Operation to transact a value.
#[derive(Clone, Copy, Debug)]
pub struct Transact;

/// [`Transact`]ed value.
pub type Transacted<T> = <T as Handler<Transact>>::Ok;

/// Operation to commiting a value.
#[derive(Clone, Copy, Debug)]
pub struct Commit;

/// Selector of `W` by `B`.
#[derive(Clone, Copy, Debug)]
pub struct By<W, B> {
    /// Type of the value to select.
    _what: PhantomData<W>,

    /// Value to select by.
    by: B,
}

impl<W, B> By<W, B> {
    /// Creates a new [`By`] with the given value.
    #[must_use]
    pub fn new(by: B) -> Self {
        Self {
            _what: PhantomData,
            by,
        }
    }

    /// Consumes this [`By`] and returns the inner value.
    #[must_use]
    pub fn into_inner(self) -> B {
        self.by
    }
}
