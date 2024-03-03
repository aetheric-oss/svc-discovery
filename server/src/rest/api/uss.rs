//! REST API for the U-Space Interfaces
//! Implements the ASTM Standard at <https://github.com/uastech/standards/blob/astm_rid_api_2.1/remoteid/canonical.yaml>

use super::rest_types::*;
use crate::grpc::client::GrpcClients;
use axum::extract::Query;
use axum::{Extension, Json};
use geo::algorithm::haversine_distance::HaversineDistance;
use hyper::StatusCode;

const MAX_DISPLAY_AREA_DIAGONAL_METERS: f64 = 7_000.0;

/// A window for a given area of interest defined by two opposite corners
struct Window {
    lon1: f64,
    lat1: f64,
    lon2: f64,
    lat2: f64,
}

impl Window {
    /// Calculate the diagonal distance of the window
    fn diagonal(&self) -> f64 {
        let p1 = geo::Point::<f64>::new(self.lon1, self.lat1);
        let p2 = geo::Point::<f64>::new(self.lon2, self.lat2);
        p1.haversine_distance(&p2)
    }
}

/// Check if there are identification service areas for a given RID
async fn check_isas(_grpc_clients: &mut GrpcClients, _window: &Window) -> Result<bool, StatusCode> {
    // TODO(R4): grpc call to svc-gis
    // with optional 'check' parameter to return no values
    Ok(false)
}

/// Get recent flights for a given area from svc-gis
async fn get_recent_flights(
    _grpc_clients: &mut GrpcClients,
    _window: &Window,
    duration_s: f32,
) -> Result<Vec<RIDFlight>, StatusCode> {
    if !(0.0..=60.0).contains(&duration_s) {
        rest_error!("(get_recent_flights) duration_s must be >= 0.0.");
        return Err(StatusCode::BAD_REQUEST);
    } else if duration_s == 0.0 {
        return Ok(vec![]);
    }

    // TODO(R4): grpc call to svc-gis
    Ok(vec![])
}

/// Parse a coordinate (float) from a string
fn parse_coordinate(coordinate: &str, lat: bool) -> Result<f64, StatusCode> {
    let Ok(value) = coordinate.parse::<f64>() else {
        rest_error!("(parse_coordinate) view must be a string of format 'lat1,lon1,lat2,lon2' with floating point values.");
        return Err(StatusCode::BAD_REQUEST);
    };

    if lat && !(-90.0..=90.0).contains(&value) {
        rest_error!("(parse_coordinate) latitude must be between -90.0 and 90.0.");
        return Err(StatusCode::BAD_REQUEST);
    } else if !lat && !(-180.0..=180.0).contains(&value) {
        rest_error!("(parse_coordinate) longitude must be between -180.0 and 180.0.");
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(value)
}

/// Validate the input for the get_flights endpoint
fn validate_get_flights_request(payload: &GetFlightsRequest) -> Result<Window, StatusCode> {
    if payload.recent_positions_duration < 0.0 || payload.recent_positions_duration > 60.0 {
        rest_error!("(validate_get_flights_request) recent_positions_duration must be >= 0.0.");
        return Err(StatusCode::BAD_REQUEST);
    }

    let values = payload.view.split(',').collect::<Vec<&str>>();
    if values.len() != 4 {
        rest_error!(
            "(validate_get_flights_request) view must be a string of format 'lat1,lon1,lat2,lon2'."
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    let window = Window {
        lat1: parse_coordinate(values[0], true)?,
        lon1: parse_coordinate(values[1], false)?,
        lat2: parse_coordinate(values[2], true)?,
        lon2: parse_coordinate(values[3], false)?,
    };

    if window.diagonal() > MAX_DISPLAY_AREA_DIAGONAL_METERS {
        rest_error!("(get_flights) The requested view rectangle was too large.");
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    Ok(window)
}

/// Get flights for a given area
#[utoipa::path(
    get,
    path = "/uss/flights",
    tag = "svc-discovery",
    request_body = GetFlightsRequest,
    responses(
        (status = 200, description = "Flight information was successfully retrieved.", body = String),
        (status = 400, description = "One or more input parameters were missing or invalid."),
        (status = 401, description = "Bearer access token was not provided in Authorization header, token could not be decoded, or token was invalid."),
        (status = 403, description = "The access token was decoded successfully but did not include a scope appropriate to this endpoint."),
        (status = 413, description = "The requested view rectangle was too large.")
    )
)]
pub async fn get_flights(
    Extension(grpc_clients): Extension<GrpcClients>,
    Query(query): Query<GetFlightsRequest>,
) -> Result<Json<GetFlightsResponse>, StatusCode> {
    rest_debug!("(get_flights) entry.");

    // TODO(R5): 403 and 401 are not implemented yet

    let window = validate_get_flights_request(&query)?;
    let response = GetFlightsResponse {
        flights: get_recent_flights(
            &mut grpc_clients.clone(),
            &window,
            query.recent_positions_duration,
        )
        .await?,
        no_isas_present: !check_isas(&mut grpc_clients.clone(), &window).await?,
        ..Default::default() // applies current timestamp
    };

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_flights_recent_positions() {
        let config = crate::config::Config::default();
        let grpc_clients = Extension(crate::grpc::client::GrpcClients::default(config));

        let request = GetFlightsRequest {
            view: "0.0,0.0,0.0".to_string(),
            recent_positions_duration: -0.0001,
        };

        let e = get_flights(grpc_clients.clone(), Query(request))
            .await
            .unwrap_err();
        assert_eq!(e, StatusCode::BAD_REQUEST);

        let request = GetFlightsRequest {
            view: "0.0,0.0,0.0".to_string(),
            recent_positions_duration: 60.0001,
        };

        let e = get_flights(grpc_clients.clone(), Query(request))
            .await
            .unwrap_err();
        assert_eq!(e, StatusCode::BAD_REQUEST);

        // Valid request
        let request = GetFlightsRequest {
            view: "0.0,0.0,0.0,0.0".to_string(),
            recent_positions_duration: 0.0,
        };
        let _ = get_flights(grpc_clients.clone(), Query(request))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_flights_view() {
        let config = crate::config::Config::default();
        let grpc_clients = Extension(crate::grpc::client::GrpcClients::default(config));

        // invalid - too many coordinates
        let request = GetFlightsRequest {
            view: "0.0,0.0,0.0,0.0,0.0".to_string(),
            recent_positions_duration: 0.0,
        };

        let e = get_flights(grpc_clients.clone(), Query(request))
            .await
            .unwrap_err();
        assert_eq!(e, StatusCode::BAD_REQUEST);

        // invalid - too few coordinates
        let request = GetFlightsRequest {
            view: "0.0,0.0,0.0".to_string(),
            recent_positions_duration: 0.0,
        };

        let e = get_flights(grpc_clients.clone(), Query(request))
            .await
            .unwrap_err();
        assert_eq!(e, StatusCode::BAD_REQUEST);

        for i in [
            "-90.01,0,0,0",
            "90.01,0,0,0",
            "0,-180.01,0,0",
            "0,180.01,0,0",
            "0,0,-90.01,0",
            "0,0,90.01,0",
            "0,0,0,-180.01",
            "0,0,0,180.01",
        ] {
            let request = GetFlightsRequest {
                view: i.to_string(),
                recent_positions_duration: 0.0,
            };

            let e = get_flights(grpc_clients.clone(), Query(request))
                .await
                .unwrap_err();
            assert_eq!(e, StatusCode::BAD_REQUEST);
        }

        // invalid - too large
        let request = GetFlightsRequest {
            view: "52.392365,4.850067,52.364510,4.959106".to_string(),
            recent_positions_duration: 0.0,
        };

        let e = get_flights(grpc_clients.clone(), Query(request))
            .await
            .unwrap_err();
        assert_eq!(e, StatusCode::PAYLOAD_TOO_LARGE);

        // valid request
        let request = GetFlightsRequest {
            view: "52.392365,4.850067,52.371385,4.906068".to_string(),
            recent_positions_duration: 0.0,
        };

        let _ = get_flights(grpc_clients.clone(), Query(request))
            .await
            .unwrap();
    }
}
