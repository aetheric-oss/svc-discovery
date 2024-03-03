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
            .add_source(Environment::default().separator("__"))
            .build()?
            .try_deserialize()
    }
}
