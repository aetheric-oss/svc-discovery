//! REST
//! provides server implementations for REST API

#[macro_use]
pub mod macros;
pub mod api;
pub mod server;

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        api::uss::get_flights,
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
struct ApiDoc;

/// Create OpenAPI3 Specification File
pub fn generate_openapi_spec(target: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = ApiDoc::openapi()
        .to_pretty_json()
        .expect("(ERROR) unable to write openapi specification to json.");

    std::fs::write(target, output).expect("(ERROR) unable to write json string to file.");

    Ok(())
}
