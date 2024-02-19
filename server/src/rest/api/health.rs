//! Rest API implementations

use crate::grpc::client::GrpcClients;
use axum::extract::Extension;
use hyper::StatusCode;

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
    Extension(mut _grpc_clients): Extension<GrpcClients>,
) -> Result<(), StatusCode> {
    rest_debug!("(health_check) entry.");

    let ok = true;

    // FIXME - uncomment this when you have a dependency
    // This health check is to verify that ALL dependencies of this
    // microservice are running.

    // let result = grpc_clients.storage.get_client().await;
    // if result.is_none() {
    //     let error_msg = "svc-storage unavailable.".to_string();
    //     rest_error!("(health_check) {}", &error_msg);
    //     ok = false;
    // };

    // let result = grpc_clients.pricing.get_client().await;
    // if result.is_none() {
    //     let error_msg = "svc-pricing unavailable.".to_string();
    //     rest_error!("(health_check) {}", &error_msg);
    //     ok = false;
    // };

    // let result = grpc_clients.scheduler.get_client().await;
    // if result.is_none() {
    //     let error_msg = "svc-scheduler unavailable.".to_string();
    //     rest_error!("(health_check) {}", &error_msg);
    //     ok = false;
    // };

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
