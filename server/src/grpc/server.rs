//! gRPC server implementation

///module generated from proto/svc-discovery-grpc.proto
pub mod grpc_server {
    #![allow(unused_qualifications, missing_docs)]
    tonic::include_proto!("grpc");
}
use grpc_server::rpc_service_server::{RpcService, RpcServiceServer};
use grpc_server::{ReadyRequest, ReadyResponse};

use crate::config::Config;
use crate::shutdown_signal;

use std::fmt::Debug;
use std::net::SocketAddr;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

/// struct to implement the gRPC server functions
#[derive(Debug, Default, Copy, Clone)]
pub struct GRPCServerImpl {}

#[tonic::async_trait]
impl RpcService for GRPCServerImpl {
    /// Returns ready:true when service is available
    async fn is_ready(
        &self,
        _request: Request<ReadyRequest>,
    ) -> Result<Response<ReadyResponse>, Status> {
        grpc_debug!("(grpc is_ready) entry.");
        let response = ReadyResponse { ready: true };
        Ok(Response::new(response))
    }
}

/// Starts the grpc servers for this microservice using the provided configuration
///
/// # Example:
/// ```
/// use svc_discovery::grpc::server::grpc_server;
/// use svc_discovery::config::Config;
/// async fn example() -> Result<(), tokio::task::JoinError> {
///     let config = Config::default();
///     tokio::spawn(grpc_server(config, None)).await
/// }
/// ```
pub async fn grpc_server(config: Config, shutdown_rx: Option<tokio::sync::oneshot::Receiver<()>>) {
    grpc_debug!("entry.");

    // GRPC Server
    let grpc_port = config.docker_port_grpc;
    let full_grpc_addr: SocketAddr = match format!("[::]:{}", grpc_port).parse() {
        Ok(addr) => addr,
        Err(e) => {
            grpc_error!("Failed to parse gRPC address: {}", e);
            return;
        }
    };

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    let imp = GRPCServerImpl::default();
    health_reporter
        .set_serving::<RpcServiceServer<GRPCServerImpl>>()
        .await;

    //start server
    grpc_info!("Starting GRPC servers on: {}.", full_grpc_addr);
    match Server::builder()
        .add_service(health_service)
        .add_service(RpcServiceServer::new(imp))
        .serve_with_shutdown(full_grpc_addr, shutdown_signal("grpc", shutdown_rx))
        .await
    {
        Ok(_) => grpc_info!("gRPC server running at: {}.", full_grpc_addr),
        Err(e) => {
            grpc_error!("could not start gRPC server: {}", e);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_grpc_server_start_and_shutdown() {
        use tokio::time::{sleep, Duration};
        lib_common::logger::get_log_handle().await;
        ut_info!("start");

        let config = Config::default();

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // Start the grpc server
        tokio::spawn(grpc_server(config, Some(shutdown_rx)));

        // Give the server time to get through the startup sequence (and thus code)
        sleep(Duration::from_secs(1)).await;

        // Shut down server
        assert!(shutdown_tx.send(()).is_ok());

        ut_info!("success");
    }
}
