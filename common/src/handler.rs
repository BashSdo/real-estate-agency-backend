//! [`Handler`] abstractions.

use std::future::Future;

/// Executable handler.
pub trait Handler<Args = ()> {
    /// Type of successful [`Handler`] result.
    type Ok;

    /// Type of this [`Handler`] error.
    type Err;

    /// Executes this [`Handler`] with the provided arguments.
    fn execute(
        &self,
        args: Args,
    ) -> impl Future<Output = Result<Self::Ok, Self::Err>>;
}
