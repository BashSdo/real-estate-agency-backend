//! Background environment for running [`Task`]s.

use std::{
    error::Error,
    future::{Future, IntoFuture},
    iter,
};

use futures::{
    future::{self, LocalBoxFuture},
    FutureExt as _, TryFutureExt as _,
};
use tokio::task;

#[cfg(doc)]
use crate::Task;

/// Background environment for running [`Task`]s.
#[derive(Debug, Default)]
pub struct Background {
    /// Local set of tasks.
    set: task::LocalSet,

    /// Handles of spawned tasks.
    handles: Vec<task::JoinHandle<Result<(), Box<dyn Error + 'static>>>>,
}

impl Background {
    /// Spawns a new [`Task`] inside the [`Background`] environment.
    pub fn spawn<F, E>(&mut self, future: F)
    where
        F: Future<Output = Result<(), E>> + 'static,
        E: Error + 'static,
    {
        self.handles.push(self.set.spawn_local(
            future.map_err(|e| Box::<dyn Error + 'static>::from(Box::new(e))),
        ));
    }
}

impl IntoFuture for Background {
    type Output = Result<(), Box<dyn Error>>;
    type IntoFuture = LocalBoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let Self { set, handles } = self;
        future::try_join_all(iter::once(set.map(Ok).boxed_local()).chain(
            handles.into_iter().map(|h| {
                h.map(|r| match r {
                    Ok(Ok(())) => Ok(()),
                    Ok(Err(e)) => Err(e),
                    Err(e) => {
                        Err(Box::<dyn Error + 'static>::from(Box::new(e)))
                    }
                })
                .boxed_local()
            }),
        ))
        .map_ok(drop)
        .boxed_local()
    }
}
