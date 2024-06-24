//! # Config
//!
//! Define and implement config options for module

use anyhow::Result;
use config::{ConfigError, Environment};
use dotenv::dotenv;
use serde::Deserialize;

/// struct holding configuration options
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// port to be used for gRPC server
    pub docker_port_grpc: u16,
    /// port to be used for REST server
    pub docker_port_rest: u16,
    /// path to log configuration YAML file
    pub log_config: String,
    /// host for the gis gRPC server
    pub gis_host_grpc: String,
    /// port for the gis gRPC server
    pub gis_port_grpc: u16,
    /// host for the DSS
    pub dss_host: String,
    /// port for the DSS
    pub dss_port: u16,
    /// host for the oauth server
    pub oauth_host: String,
    /// port for the oauth server
    pub oauth_port: u16,
    /// Rate limit - requests per second for REST requests
    pub rest_request_limit_per_second: u8,
    /// Enforces a limit on the concurrent number of requests the underlying service can handle
    pub rest_concurrency_limit_per_service: u8,
    /// Full url (including port number) to be allowed as request origin for
    /// REST requests
    pub rest_cors_allowed_origin: String,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Default values for Config
    pub fn new() -> Self {
        Config {
            docker_port_grpc: 50051,
            docker_port_rest: 8000,
            log_config: String::from("log4rs.yaml"),
            gis_host_grpc: String::from("localhost"),
            gis_port_grpc: 50052,
            dss_host: String::from("localhost"),
            dss_port: 50054,
            oauth_host: String::from("localhost"),
            oauth_port: 50053,
            rest_request_limit_per_second: 2,
            rest_concurrency_limit_per_service: 5,
            rest_cors_allowed_origin: String::from("http://localhost:3000"),
        }
    }

    /// Create a new `Config` object using environment variables
    pub fn try_from_env() -> Result<Self, ConfigError> {
        // read .env file if present
        dotenv().ok();
        let default_config = Config::default();

        config::Config::builder()
            .set_default("docker_port_grpc", default_config.docker_port_grpc)?
            .set_default("docker_port_rest", default_config.docker_port_rest)?
            .set_default("log_config", default_config.log_config)?
            .set_default("gis_host_grpc", default_config.gis_host_grpc)?
            .set_default("gis_port_grpc", default_config.gis_port_grpc)?
            .set_default("dss_host", default_config.dss_host)?
            .set_default("dss_port", default_config.dss_port)?
            .set_default("oauth_host", default_config.oauth_host)?
            .set_default("oauth_port", default_config.oauth_port)?
            .set_default(
                "rest_concurrency_limit_per_service",
                default_config.rest_concurrency_limit_per_service,
            )?
            .set_default(
                "rest_request_limit_per_seconds",
                default_config.rest_request_limit_per_second,
            )?
            .set_default(
                "rest_cors_allowed_origin",
                default_config.rest_cors_allowed_origin,
            )?
            .add_source(Environment::default().separator("__"))
            .build()?
            .try_deserialize()
    }
}
