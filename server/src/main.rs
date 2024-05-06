//! Main function starting the server and initializing dependencies.

use clap::Parser;
use grpc::server::grpc_server;
use lib_common::logger::load_logger_config_from_file;
use log::info;
use rest::{generate_openapi_spec, server::rest_server, ApiDoc};
use svc_discovery::config::Config;
use svc_discovery::grpc;
use svc_discovery::rest;
use svc_discovery::Cli;

/// Main entry point: starts gRPC Server on specified address and port
#[tokio::main]
#[cfg(not(tarpaulin_include))]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Will use default config settings if no environment vars are found.
    let config = Config::try_from_env()
        .map_err(|e| format!("Failed to load configuration from environment: {}", e))?;

    // Try to load log configuration from the provided log file.
    // Will default to stdout debug logging if the file can not be loaded.
    load_logger_config_from_file(config.log_config.as_str())
        .await
        .or_else(|e| Ok::<(), String>(log::error!("(main) {}", e)))?;

    // Allow option to only generate the spec file to a given location
    // locally: cargo run -- --api ./out/$(PACKAGE_NAME)-openapi.json
    // or `make rust-openapi` and `make rust-validate-openapi`
    let args = Cli::parse();
    if let Some(target) = args.openapi {
        return generate_openapi_spec::<ApiDoc>(&target).map_err(|e| e.into());
    }

    // Start REST server
    tokio::spawn(rest_server(config.clone()));

    // Start gRPC server
    let _ = tokio::spawn(grpc_server(config)).await;

    info!("Server shutdown.");

    // Make sure all log message are written/ displayed before shutdown
    log::logger().flush();

    Ok(())
}
