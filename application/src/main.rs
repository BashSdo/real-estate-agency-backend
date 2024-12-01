use std::{
    future::IntoFuture as _,
    io,
    sync::{Arc, OnceLock},
    time,
};

use application::{api, graphql, subscriptions, Args, Config};
use axum::{
    extract::MatchedPath,
    routing::{get, on, MethodFilter},
    Extension, Router,
};
use axum_client_ip::InsecureClientIp;
use futures::{future, TryFutureExt as _};
use service::{
    infra::{postgres, Postgres},
    Service,
};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing as log;
use tracing_subscriber::{
    filter::filter_fn,
    layer::{Layer as _, SubscriberExt as _},
    util::SubscriberInitExt as _,
};

const STDERR_LEVELS: &[log::Level] = &[log::Level::WARN, log::Level::ERROR];

static LOG_LEVEL: OnceLock<log::Level> = OnceLock::new();

postgres::embed_migrations!("../migrations");

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_ansi(true)
                .with_thread_names(true)
                .with_writer(io::stdout)
                .with_filter(filter_fn(|meta| {
                    meta.is_span()
                        || (!STDERR_LEVELS.contains(meta.level()))
                            && LOG_LEVEL
                                .get()
                                .copied()
                                .unwrap_or(log::Level::INFO)
                                >= *meta.level()
                })),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_ansi(true)
                .with_thread_names(true)
                .with_writer(io::stderr)
                .with_filter(filter_fn(|meta| {
                    meta.is_span()
                        || (STDERR_LEVELS.contains(meta.level()))
                            && LOG_LEVEL
                                .get()
                                .copied()
                                .unwrap_or(log::Level::INFO)
                                >= *meta.level()
                })),
        )
        .init();

    _ = start().await;
}

async fn start() -> Result<(), ()> {
    let Args { config } = Args::parse().map_err(|e| {
        log::error!("failed to parse command line arguments: {e}");
    })?;

    let Config {
        postgres,
        service,
        server,
        log,
    } = Config::new(config).map_err(|e| {
        log::error!("failed to load `Config`: {e}");
    })?;

    LOG_LEVEL
        .set(log.level.into())
        .unwrap_or_else(|_| unreachable!("first initialization"));

    let postgres_config = postgres.into();
    let mut postgres = Postgres::new(&postgres_config).map_err(|e| {
        log::error!("failed to initialize `Postgres` client: {e}");
    })?;

    migrations::runner()
        .run_async(&mut postgres)
        .await
        .map_err(|e| {
            log::error!("failed to run database migrations: {e}");
        })?;

    let (service, background) = Service::new(service.into(), postgres);

    let schema = api::Schema::new(api::Query, api::Mutation, api::Subscription);

    let mut cors = CorsLayer::new()
        .allow_methods([
            http::Method::GET,
            http::Method::OPTIONS,
            http::Method::POST,
        ])
        .allow_headers([
            http::header::AUTHORIZATION,
            http::header::CONTENT_TYPE,
        ]);
    for origin in server.cors.origins {
        cors = cors.allow_origin(
            origin.parse::<http::header::HeaderValue>().map_err(|e| {
                log::error!("`{origin}` is not current CORS origin: {e}");
            })?,
        );
    }

    let app = Router::new()
        .route(
            "/graphql",
            on(MethodFilter::GET.or(MethodFilter::POST), graphql),
        )
        .route("/subscriptions", get(subscriptions))
        .layer(Extension(Arc::new(schema)))
        .layer(Extension(service))
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|r: &http::Request<_>| {
                    tracing::info_span!(
                        "HTTP request",
                        http.client_ip = InsecureClientIp::from(
                            r.headers(),
                            r.extensions()
                        )
                            .map(|ip| ip.0.to_string())
                            .ok(),
                        http.flavor = ?r.version(),
                        http.host = r.uri().host(),
                        http.method = r.method().as_str(),
                        http.route = r
                            .extensions()
                            .get::<MatchedPath>()
                            .map(MatchedPath::as_str),
                        http.scheme = r
                            .uri()
                            .scheme()
                            .map(http::uri::Scheme::as_str),
                        http.target = r
                            .uri()
                            .path_and_query()
                            .map(http::uri::PathAndQuery::as_str),
                        http.user_agent = r
                            .headers()
                            .get("User-Agent")
                            .and_then(|h| h.to_str().ok()),
                        http.status_code = tracing::field::Empty,
                    )
                })
                .on_response(
                    |r: &http::Response<_>,
                     dur: time::Duration,
                     span: &tracing::Span| {
                        span.record(
                            "http.status_code",
                            tracing::field::display(r.status().as_u16()),
                        );

                        if r.status().is_server_error()
                            || r.status().is_client_error()
                        {
                            tracing::error!(
                                duration = format!("{}ms", dur.as_millis()),
                            );
                        } else {
                            tracing::info!(
                                duration = format!("{}ms", dur.as_millis()),
                            );
                        }
                    },
                ),
        );

    let listener = TcpListener::bind((server.host.clone(), server.port))
        .await
        .map_err(|e| {
            log::error!(
                "failed to listen on `{}:{}`: {e}",
                server.host,
                server.port,
            );
        })?;

    log::info!("listening on `{}:{}`", server.host, server.port);

    let serve = axum::serve(listener, app);

    future::try_join(
        serve
            .into_future()
            .map_err(|e| log::error!("webserver failed: {e}")),
        background.into_future().map_err(|e| {
            log::error!("background task failed: {e}");
        }),
    )
    .await
    .map(drop)
}
