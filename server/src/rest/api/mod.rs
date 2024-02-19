//! REST API for the discovery service

pub mod health;
pub mod uss;

/// openapi generated rest types
pub mod rest_types {
    include!("../../../../openapi/types.rs");
}
