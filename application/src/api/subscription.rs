//! GraphQL [`Subscription`]s definitions.

use common::DateTime;
use futures::{
    stream::{self, BoxStream},
    FutureExt as _, StreamExt as _,
};
use juniper::graphql_subscription;

use crate::{context, Context, Error};

/// Root of all GraphQL subscription.
#[derive(Clone, Copy, Debug)]
pub struct Subscription;

#[graphql_subscription(context = Context)]
impl Subscription {
    /// Subscription waiting for the current authenticated session to expire.
    ///
    /// # Errors
    ///
    /// Possible error codes:
    /// - `AUTHORIZATION_REQUIRED` - if the current session is not
    ///                              authenticated or session expired.
    pub async fn wait_session(
        &self,
        ctx: &Context,
    ) -> Result<BoxStream<'static, Result<bool, Error>>, Error> {
        let session = ctx.current_session().await?;
        let timeout = session.expires_at - DateTime::now();
        Ok(stream::once(
            tokio::time::sleep(timeout).map(|()| {
                Err(context::AuthError::AuthroizationRequired.into())
            }),
        )
        .boxed())
    }
}
