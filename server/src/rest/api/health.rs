//! Rest API implementations

use crate::grpc::client::GrpcClients;
use axum::extract::Extension;
use hyper::StatusCode;
use svc_gis_client_grpc::client::ReadyRequest;
use svc_gis_client_grpc::prelude::GisServiceClient;

/// Provides a way to tell a caller if the service is healthy.
/// Checks dependencies, making sure all connections can be made.
#[utoipa::path(
    get,
    path = "/health",
    tag = "svc-discovery",
    responses(
        (status = 200, description = "Service is healthy, all dependencies running."),
        (status = 503, description = "Service is unhealthy, one or more dependencies unavailable.")
    )
)]
pub async fn health_check(
    Extension(grpc_clients): Extension<GrpcClients>,
) -> Result<(), StatusCode> {
    rest_debug!("(health_check) entry.");

    let mut ok = true;

    if grpc_clients.gis.is_ready(ReadyRequest {}).await.is_err() {
        rest_error!("(health_check) svc-gis client unavailable.");
        ok = false;
    };

    match ok {
        true => {
            rest_debug!("(health_check) healthy, all dependencies running.");
            Ok(())
        }
        false => {
            rest_error!("(health_check) unhealthy, 1+ dependencies down.");
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let config = crate::config::Config::default();
        let clients = GrpcClients::default(config);
        let result = health_check(Extension(clients)).await;
        assert!(result.is_ok());
    }
}
