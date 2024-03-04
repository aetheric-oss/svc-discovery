//! Rest server implementation

use super::api;
use crate::config::Config;
use crate::grpc::client::GrpcClients;
use crate::shutdown_signal;
use axum::{
    error_handling::HandleErrorLayer,
    extract::Extension,
    http::{HeaderValue, StatusCode},
    routing, BoxError, Router,
};
use std::net::SocketAddr;
use tower::{
    buffer::BufferLayer,
    limit::{ConcurrencyLimitLayer, RateLimitLayer},
    ServiceBuilder,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// Starts the REST API server for this microservice
#[cfg(not(tarpaulin_include))]
pub async fn rest_server(config: Config) -> Result<(), ()> {
    rest_info!("(rest_server) entry.");
    let rest_port = config.docker_port_rest;
    let full_rest_addr: SocketAddr = format!("[::]:{}", rest_port).parse().map_err(|e| {
        rest_error!("(rest_server) invalid address: {:?}, exiting.", e);
    })?;

    let cors_allowed_origin = config
        .rest_cors_allowed_origin
        .parse::<HeaderValue>()
        .map_err(|e| {
            rest_error!(
                "(rest_server) invalid cors_allowed_origin address: {:?}, exiting.",
                e
            );
        })?;

    // Rate limiting
    let rate_limit = config.rest_request_limit_per_second as u64;
    let concurrency_limit = config.rest_concurrency_limit_per_service as usize;
    let limit_middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            rest_warn!("(server) too many requests: {}", e);
            (
                StatusCode::TOO_MANY_REQUESTS,
                "(server) too many requests.".to_string(),
            )
        }))
        .layer(BufferLayer::new(100))
        .layer(ConcurrencyLimitLayer::new(concurrency_limit))
        .layer(RateLimitLayer::new(
            rate_limit,
            std::time::Duration::from_secs(1),
        ));

    rest_debug!("(rest_server) entry.");
    let grpc_clients = GrpcClients::default(config.clone());
    let app = Router::new()
        .route("/health", routing::get(api::health::health_check)) // MUST HAVE
        .route("/uss/flights", routing::get(api::uss::get_flights))
        // .route(
        //     "/uss/identification_service_areas/:id",
        //     routing::get(api::get_isas),
        // )
        // .route(
        //     "/uss/flights/:id/details",
        //     routing::get(api::get_flight_details),
        // )
        .layer(
            CorsLayer::new()
                .allow_origin(cors_allowed_origin)
                .allow_headers(Any)
                .allow_methods(Any),
        )
        .layer(limit_middleware)
        .layer(Extension(grpc_clients)); // Extension layer must be last

    rest_info!("(rest) hosted at {:?}", full_rest_addr);
    axum::Server::bind(&full_rest_addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal("rest"))
        .await
        .map_err(|e| {
            rest_error!("could not start REST server: {}", e);
        })?;

    Ok(())
}
