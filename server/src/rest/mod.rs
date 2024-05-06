//! REST
//! provides server implementations for REST API

#[macro_use]
pub mod macros;
pub mod api;
pub mod server;

use std::fmt::{self, Display, Formatter};
use utoipa::OpenApi;

/// OpenAPI 3.0 specification for this service
#[derive(OpenApi, Copy, Clone, Debug)]
#[openapi(
    paths(
        api::uss::get_flights,
        api::uss::demo_flights
    ),
    components(
        schemas(
            api::rest_types::GetFlightsRequest,
            api::rest_types::GetFlightsResponse,
            api::rest_types::Time,
            api::rest_types::Altitude,
            api::rest_types::LatLngPoint,
            api::rest_types::Radius,
            api::rest_types::Circle,
            api::rest_types::Polygon,
            api::rest_types::Volume3D,
            api::rest_types::Volume4D,
            api::rest_types::RIDAircraftState,
            api::rest_types::OperatingArea,
            api::rest_types::RIDHeight,
            api::rest_types::RIDAircraftPosition,
            api::rest_types::RIDRecentAircraftPosition,
            api::rest_types::RIDFlight,
            api::rest_types::UAType,
            api::rest_types::RIDOperationalStatus,
            api::rest_types::SpeedAccuracy,
            api::rest_types::HorizontalAccuracy,
            api::rest_types::VerticalAccuracy,
            api::rest_types::RIDHeightReference,
        )
    ),
    tags(
        (name = "svc-discovery", description = "svc-discovery REST API")
    )
)]
#[cfg(not(tarpaulin_include))]
pub struct ApiDoc;

/// Errors with OpenAPI generation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpenApiError {
    /// Failed to export as JSON string
    Json,

    /// Failed to write to file
    FileWrite,
}

impl std::error::Error for OpenApiError {}

impl Display for OpenApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OpenApiError::Json => write!(f, "Failed to export as JSON string"),
            OpenApiError::FileWrite => write!(f, "Failed to write to file"),
        }
    }
}

/// Create OpenAPI 3.0 Specification File
pub fn generate_openapi_spec<T>(target: &str) -> Result<(), OpenApiError>
where
    T: OpenApi,
{
    let output = T::openapi().to_pretty_json().map_err(|e| {
        rest_error!("(generate_openapi_spec) failed to export as JSON string: {e}");
        OpenApiError::Json
    })?;

    std::fs::write(target, output).map_err(|e| {
        rest_error!("(generate_openapi_spec) failed to write to file: {e}");
        OpenApiError::FileWrite
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_openapi_spec() {
        let target = "/nonsense/";
        let error = generate_openapi_spec::<ApiDoc>(target).unwrap_err();
        assert_eq!(error, OpenApiError::FileWrite);

        // TODO(R5): Is it possible to make the JSON export fail?
        // #[derive(OpenApi)]
        // #[openapi(
        //     paths(invalid)
        // )]
        // struct InvalidApi;
        // let error = generate_openapi_spec::<InvalidApi>("test.json").unwrap_err();
        // assert_eq!(error, OpenApiError::Json);
    }
}
