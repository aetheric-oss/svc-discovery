//! REST API for the U-Space Interfaces
//! Implements the ASTM Standard at <https://github.com/uastech/standards/blob/astm_rid_api_2.1/remoteid/canonical.yaml>

use super::rest_types::*;
use crate::grpc::client::GrpcClients;
use axum::extract::Query;
use axum::{Extension, Json};
use chrono::{DateTime, Duration, Utc};
use geo::algorithm::haversine_distance::HaversineDistance;
use hyper::StatusCode;
use num_traits::FromPrimitive;
use svc_gis_client_grpc::client::GetFlightsRequest as GisFlightsRequest;
use svc_gis_client_grpc::prelude::AircraftType;
use svc_gis_client_grpc::prelude::GisServiceClient;
use svc_gis_client_grpc::prelude::OperationalStatus;
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

impl From<AircraftType> for UAType {
    fn from(t: AircraftType) -> Self {
        match t {
            AircraftType::Undeclared => UAType::NotDeclared,
            AircraftType::Aeroplane => UAType::Aeroplane,
            AircraftType::Glider => UAType::Glider,
            AircraftType::Gyroplane => UAType::Gyroplane,
            AircraftType::Ornithopter => UAType::Ornithopter,
            AircraftType::Hybridlift => UAType::HybridLift,
            AircraftType::Kite => UAType::Kite,
            AircraftType::Freeballoon => UAType::FreeBalloon,
            AircraftType::Captiveballoon => UAType::CaptiveBalloon,
            AircraftType::Airship => UAType::Airship,
            AircraftType::Rocket => UAType::Rocket,
            AircraftType::Tethered => UAType::TetheredPoweredAircraft,
            AircraftType::Groundobstacle => UAType::GroundObstacle,
            AircraftType::Other => UAType::Other,
            AircraftType::Unpowered => UAType::FreeFallOrParachute,
            AircraftType::Rotorcraft => UAType::Helicopter, // includes multirotor
        }
    }
}

impl From<OperationalStatus> for RIDOperationalStatus {
    fn from(s: OperationalStatus) -> Self {
        match s {
            OperationalStatus::Undeclared => RIDOperationalStatus::Undeclared,
            OperationalStatus::Ground => RIDOperationalStatus::Ground,
            OperationalStatus::Airborne => RIDOperationalStatus::Airborne,
            OperationalStatus::Emergency => RIDOperationalStatus::Emergency,
            OperationalStatus::RemoteIdSystemFailure => RIDOperationalStatus::RemoteIDSystemFailure,
        }
    }
}

impl TryFrom<svc_gis_client_grpc::client::AircraftState> for RIDAircraftState {
    type Error = StatusCode;

    fn try_from(state: svc_gis_client_grpc::client::AircraftState) -> Result<Self, Self::Error> {
        let timestamp: DateTime<Utc> = match state.timestamp {
            Some(t) => t.into(),
            None => {
                rest_error!("(RIDAircraftState::try_from) timestamp is required.");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let Some(position) = state.position else {
            rest_error!("(RIDAircraftState::try_from) position is required.");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };

        let status: OperationalStatus = match FromPrimitive::from_i32(state.status) {
            Some(s) => s,
            None => {
                rest_error!("(RIDAircraftState::try_from) status is required.");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        Ok(RIDAircraftState {
            timestamp: Time {
                value: timestamp.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                ..Default::default()
            },
            timestamp_accuracy: 0.0,
            operational_status: status.into(),
            position: RIDAircraftPosition {
                lat: position.latitude,
                lng: position.longitude,
                alt: position.altitude_meters,
                accuracy_h: HorizontalAccuracy::HAUnknown,
                accuracy_v: VerticalAccuracy::VAUnknown,
                extrapolated: false,
                pressure_alt: position.altitude_meters,
                height: RIDHeight {
                    distance: position.altitude_meters,
                    reference: RIDHeightReference::GroundLevel,
                },
            },
            track: state.track_angle_degrees,
            speed: state.ground_speed_mps,
            speed_accuracy: SpeedAccuracy::SAUnknown,
            vertical_speed: state.vertical_speed_mps,
        })
    }
}

impl TryFrom<svc_gis_client_grpc::client::TimePosition> for RIDRecentAircraftPosition {
    type Error = StatusCode;

    fn try_from(position: svc_gis_client_grpc::client::TimePosition) -> Result<Self, Self::Error> {
        let timestamp: DateTime<Utc> = match position.timestamp {
            Some(t) => t.into(),
            None => {
                rest_error!("(RIDRecentAircraftPosition::try_from) timestamp is required.");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let Some(position) = position.position else {
            rest_error!("(RIDAircraftPosition::try_from) position is required.");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };

        Ok(RIDRecentAircraftPosition {
            time: Time {
                value: timestamp.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                ..Default::default()
            },
            position: RIDAircraftPosition {
                lat: position.latitude,
                lng: position.longitude,
                alt: position.altitude_meters,
                accuracy_h: HorizontalAccuracy::HAUnknown,
                accuracy_v: VerticalAccuracy::VAUnknown,
                extrapolated: false,
                pressure_alt: position.altitude_meters,
                height: RIDHeight {
                    distance: position.altitude_meters,
                    reference: RIDHeightReference::GroundLevel,
                },
            },
        })
    }
}

/// Get recent flights for a given area from svc-gis
async fn get_recent_flights(
    grpc_clients: &mut GrpcClients,
    window: &Window,
    duration_s: f32,
) -> Result<Vec<RIDFlight>, StatusCode> {
    if !(0.0..=60.0).contains(&duration_s) {
        rest_error!("(get_recent_flights) duration_s must be >= 0.0.");
        return Err(StatusCode::BAD_REQUEST);
    } else if duration_s == 0.0 {
        return Ok(vec![]);
    }

    let time_start = Utc::now() - Duration::milliseconds((duration_s * 1000.0) as i64);
    let time_end = Utc::now();
    let request = GisFlightsRequest {
        window_min_x: window.lon1,
        window_min_y: window.lat1,
        window_max_x: window.lon2,
        window_max_y: window.lat2,
        time_start: Some(time_start.into()),
        time_end: Some(time_end.into()),
    };

    grpc_clients
        .gis
        .get_flights(request)
        .await
        .map_err(|e| {
            rest_error!("(get_recent_flights) gRPC call to svc-gis failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_inner()
        .flights
        .into_iter()
        .map(|f| {
            let Some(state) = f.state else {
                rest_error!("(get_recent_flights) state is required.");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            };

            let aircraft_type: AircraftType = match FromPrimitive::from_i32(f.aircraft_type) {
                Some(t) => t,
                None => {
                    rest_error!(
                        "(get_recent_flights) aircraft_type not recognized, using NotDeclared."
                    );
                    AircraftType::Undeclared
                }
            };

            Ok(RIDFlight {
                id: f.identifier.unwrap_or("".to_string()),
                aircraft_type: aircraft_type.into(),
                operating_area: OperatingArea {
                    aircraft_count: 0, // TODO(R5)
                    volumes: vec![],   // TODO(R5)
                },
                simulated: f.simulated,
                current_state: state.try_into()?,
                recent_positions: f
                    .positions
                    .into_iter()
                    .map(RIDRecentAircraftPosition::try_from)
                    .collect::<Result<Vec<_>, _>>()?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
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
