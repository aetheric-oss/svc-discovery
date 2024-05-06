//! gRPC client helpers implementation

use svc_gis_client_grpc::prelude::Client;
use svc_gis_client_grpc::prelude::GisClient;
use tokio::sync::OnceCell;

pub(crate) static CLIENTS: OnceCell<GrpcClients> = OnceCell::const_new();

/// Returns CLIENTS, a GrpcClients object with default values.
/// Uses host and port configurations using a Config object generated from
/// environment variables.
/// Initializes CLIENTS if it hasn't been initialized yet.
pub async fn get_clients() -> &'static GrpcClients {
    CLIENTS
        .get_or_init(|| async move {
            let config = crate::Config::try_from_env().unwrap_or_default();
            GrpcClients::default(config)
        })
        .await
}

/// Struct to hold all gRPC client connections
#[derive(Clone, Debug)]
#[allow(missing_copy_implementations)]
pub struct GrpcClients {
    /// A GrpcClient provided by the svc_gis_client_grpc module
    pub gis: GisClient,
}

impl GrpcClients {
    /// Create new GrpcClients with defaults
    pub fn default(config: crate::config::Config) -> Self {
        GrpcClients {
            gis: GisClient::new_client(&config.gis_host_grpc, config.gis_port_grpc, "gis"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_common::logger::get_log_handle;
    use svc_gis_client_grpc::prelude::Client as GisClient;

    #[tokio::test]
    async fn test_grpc_clients_default() {
        get_log_handle().await;
        ut_info!("(test_grpc_clients_default) Start.");

        let clients = get_clients().await;

        let gis = &clients.gis;
        ut_debug!("(test_grpc_clients_default) gis: {:?}", gis);
        assert_eq!(gis.get_name(), "gis");

        ut_info!("(test_grpc_clients_default) Success.");
    }
}
