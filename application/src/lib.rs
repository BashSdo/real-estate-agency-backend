//! Application provides API for interacting with the [`Service`].

#![deny(
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::all,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code
)]
#![forbid(non_ascii_idents)]
#![warn(
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    clippy::pedantic,
    clippy::wildcard_enum_match_arm,
    deprecated_in_future,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    unused_crate_dependencies,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]

pub mod api;
pub mod args;
pub mod config;
mod context;
pub mod error;

use std::sync::Arc;

use axum::{
    extract::WebSocketUpgrade,
    response::{IntoResponse, Response},
    Extension, Json,
};
use derive_more::Debug;
use juniper::{http::GraphQLBatchResponse, DefaultScalarValue, ScalarValue};
use juniper_axum::{extract::JuniperRequest, subscriptions};
use juniper_graphql_ws::ConnectionConfig;
// Used in binary.
use axum_client_ip as _;
use refinery as _;
use tower_http as _;
use tracing_subscriber as _;

pub use self::{
    args::Args,
    config::Config,
    context::{Context, Session},
    error::{AsError, Error},
};

/// [`Service`] with filled infrastructure dependencies.
///
/// [`Service`]: service::Service
pub type Service = service::Service<service::infra::Postgres>;

/// [`juniper`] GraphQL response.
#[derive(Debug)]
pub struct JuniperResponse<S = DefaultScalarValue>
where
    S: ScalarValue,
{
    /// Status code of the response.
    pub status_code: http::StatusCode,

    /// Underlying GraphQL response.
    #[debug(skip)]
    pub response: GraphQLBatchResponse<S>,
}

impl<S> IntoResponse for JuniperResponse<S>
where
    S: ScalarValue,
{
    fn into_response(self) -> Response {
        let Self {
            status_code,
            response,
        } = self;

        if response.is_ok() {
            Json(response).into_response()
        } else {
            (status_code, Json(response)).into_response()
        }
    }
}

/// GraphQL API handler.
pub async fn graphql(
    Extension(schema): Extension<Arc<api::Schema>>,
    context: Context,
    JuniperRequest(gql_request): JuniperRequest,
) -> JuniperResponse {
    JuniperResponse {
        status_code: context.error_status_code(),
        response: gql_request.execute(&*schema, &context).await,
    }
}

/// GraphQL subscriptions handler.
#[expect(
    clippy::unused_async,
    reason = "`async` is required to match signature"
)]
pub async fn subscriptions(
    Extension(schema): Extension<Arc<api::Schema>>,
    mut context: Context,
    ws: WebSocketUpgrade,
) -> Response {
    ws.protocols(["graphql-transport-ws", "graphql-ws"])
        .max_frame_size(1024)
        .max_message_size(1024)
        .write_buffer_size(512)
        .max_write_buffer_size(1024)
        .on_upgrade(move |socket| {
            subscriptions::serve_ws(socket, schema, move |vars| async move {
                context.apply_subscription_variables(&vars).map(|()| {
                    ConnectionConfig::new(context)
                        .with_max_in_flight_operations(10)
                })
            })
        })
}
