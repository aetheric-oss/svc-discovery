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
        let timestamp: DateTime<Utc> = state
            .timestamp
            .ok_or_else(|| {
                rest_error!("(RIDAircraftState::try_from) timestamp is required.");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .into();

        let position = state.position.ok_or_else(|| {
            rest_error!("(RIDAircraftState::try_from) position is missing.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let status: OperationalStatus = FromPrimitive::from_i32(state.status).ok_or_else(|| {
            rest_error!("(RIDAircraftState::try_from) status is required.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

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
        let timestamp: DateTime<Utc> = position
            .timestamp
            .ok_or_else(|| {
                rest_error!("(RIDRecentAircraftPosition::try_from) timestamp is required.");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .into();

        let position = position.position.ok_or_else(|| {
            rest_error!("(RIDAircraftPosition::try_from) position is missing.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let result = RIDRecentAircraftPosition {
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
        };

        Ok(result)
    }
}

impl TryFrom<svc_gis_client_grpc::client::Flight> for RIDFlight {
    type Error = StatusCode;

    fn try_from(f: svc_gis_client_grpc::client::Flight) -> Result<Self, Self::Error> {
        let state = f.state.ok_or_else(|| {
            rest_error!("(RIDFlight::try_from) state is required.");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let aircraft_type: AircraftType = match FromPrimitive::from_i32(f.aircraft_type) {
            Some(t) => t,
            None => {
                rest_warn!(
                    "(RIDFlight::try_from) aircraft_type not recognized, using NotDeclared."
                );
                AircraftType::Undeclared
            }
        };

        Ok(RIDFlight {
            id: f
                .session_id
                .unwrap_or(f.aircraft_id.unwrap_or("UNK".to_string())),
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

    if window.lat1.abs() > 90.0
        || window.lat2.abs() > 90.0
        || window.lon1.abs() > 180.0
        || window.lon2.abs() > 180.0
    {
        rest_error!("(get_recent_flights) Invalid window coordinates.");
        return Err(StatusCode::BAD_REQUEST);
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

    let flights = grpc_clients
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
        .map(TryInto::<RIDFlight>::try_into)
        .collect::<Result<Vec<_>, _>>()?;

    rest_debug!("(get_recent_flights) returning {} flights.", flights.len());
    Ok(flights)
}

/// Parse a coordinate (float) from a string
fn parse_coordinate(coordinate: &str, lat: bool) -> Result<f64, StatusCode> {
    let value = coordinate.parse::<f64>().map_err(|e| {
        rest_error!("(parse_coordinate) view must be a string of format 'lat1,lon1,lat2,lon2' with floating point values: {:?}", e);
        StatusCode::BAD_REQUEST
    })?;

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
fn validate_get_flights_request(
    payload: &GetFlightsRequest,
    diagonal_limit_meters: Option<f64>,
) -> Result<Window, StatusCode> {
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

    if let Some(limit) = diagonal_limit_meters {
        let diagonal = window.diagonal();
        // rest_debug!(
        //     "(validate_get_flights_request) diagonal: {}, limit: {}",
        //     diagonal,
        //     limit
        // );
        if diagonal > limit {
            rest_error!("(get_flights) The requested view rectangle was too large.");
            return Err(StatusCode::PAYLOAD_TOO_LARGE);
        }
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

    let window = validate_get_flights_request(&query, Some(MAX_DISPLAY_AREA_DIAGONAL_METERS))?;
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

/// Get flights for a given area
#[utoipa::path(
    get,
    path = "/demo/flights",
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
pub async fn demo_flights(
    Extension(grpc_clients): Extension<GrpcClients>,
    Query(query): Query<GetFlightsRequest>,
) -> Result<Json<GetFlightsResponse>, StatusCode> {
    rest_debug!("(get_flights) entry.");

    // TODO(R5): 403 and 401 are not implemented yet

    let window = validate_get_flights_request(&query, None)?;
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
    use svc_gis_client_grpc::client::PointZ;

    #[test]
    fn test_from_uatype() {
        assert_eq!(UAType::from(AircraftType::Undeclared), UAType::NotDeclared);
        assert_eq!(UAType::from(AircraftType::Aeroplane), UAType::Aeroplane);
        assert_eq!(UAType::from(AircraftType::Glider), UAType::Glider);
        assert_eq!(UAType::from(AircraftType::Gyroplane), UAType::Gyroplane);
        assert_eq!(UAType::from(AircraftType::Ornithopter), UAType::Ornithopter);
        assert_eq!(UAType::from(AircraftType::Hybridlift), UAType::HybridLift);
        assert_eq!(UAType::from(AircraftType::Kite), UAType::Kite);
        assert_eq!(UAType::from(AircraftType::Freeballoon), UAType::FreeBalloon);
        assert_eq!(
            UAType::from(AircraftType::Captiveballoon),
            UAType::CaptiveBalloon
        );
        assert_eq!(UAType::from(AircraftType::Airship), UAType::Airship);
        assert_eq!(UAType::from(AircraftType::Rocket), UAType::Rocket);
        assert_eq!(
            UAType::from(AircraftType::Tethered),
            UAType::TetheredPoweredAircraft
        );
        assert_eq!(
            UAType::from(AircraftType::Groundobstacle),
            UAType::GroundObstacle
        );
        assert_eq!(UAType::from(AircraftType::Other), UAType::Other);
        assert_eq!(
            UAType::from(AircraftType::Unpowered),
            UAType::FreeFallOrParachute
        );
        assert_eq!(UAType::from(AircraftType::Rotorcraft), UAType::Helicopter);
    }

    #[test]
    fn from_operational_status() {
        assert_eq!(
            RIDOperationalStatus::from(OperationalStatus::Undeclared),
            RIDOperationalStatus::Undeclared
        );
        assert_eq!(
            RIDOperationalStatus::from(OperationalStatus::Ground),
            RIDOperationalStatus::Ground
        );
        assert_eq!(
            RIDOperationalStatus::from(OperationalStatus::Airborne),
            RIDOperationalStatus::Airborne
        );
        assert_eq!(
            RIDOperationalStatus::from(OperationalStatus::Emergency),
            RIDOperationalStatus::Emergency
        );
        assert_eq!(
            RIDOperationalStatus::from(OperationalStatus::RemoteIdSystemFailure),
            RIDOperationalStatus::RemoteIDSystemFailure
        );
    }

    #[test]
    fn test_from_aircraft_state() {
        let expected_point = PointZ {
            latitude: rand::random(),
            longitude: rand::random(),
            altitude_meters: rand::random(),
        };

        let mut state = svc_gis_client_grpc::client::AircraftState {
            timestamp: Some(Utc::now().into()),
            status: 0,
            position: Some(expected_point.clone()),
            track_angle_degrees: rand::random(),
            ground_speed_mps: rand::random(),
            vertical_speed_mps: rand::random(),
        };

        let _: RIDAircraftState = state.clone().try_into().unwrap();

        state.timestamp = None;
        let error = TryInto::<RIDAircraftState>::try_into(state.clone()).unwrap_err();
        assert_eq!(error, StatusCode::INTERNAL_SERVER_ERROR);

        let now = Utc::now();
        state.timestamp = Some(now.into());

        state.position = None;
        let error = TryInto::<RIDAircraftState>::try_into(state.clone()).unwrap_err();
        assert_eq!(error, StatusCode::INTERNAL_SERVER_ERROR);
        state.position = Some(expected_point.clone());

        state.status = 100;
        let error = TryInto::<RIDAircraftState>::try_into(state.clone()).unwrap_err();
        assert_eq!(error, StatusCode::INTERNAL_SERVER_ERROR);

        state.status = 0;

        let result: RIDAircraftState = state.clone().try_into().unwrap();
        assert_eq!(
            result.timestamp.value,
            now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        );
        assert_eq!(result.operational_status, RIDOperationalStatus::Undeclared);
        assert_eq!(result.position.lat, expected_point.latitude);
        assert_eq!(result.position.lng, expected_point.longitude);
        assert_eq!(result.position.alt, expected_point.altitude_meters);
        assert_eq!(result.position.accuracy_h, HorizontalAccuracy::HAUnknown);
        assert_eq!(result.position.accuracy_v, VerticalAccuracy::VAUnknown);
        assert_eq!(result.position.extrapolated, false);
        assert_eq!(result.position.pressure_alt, expected_point.altitude_meters);
        assert_eq!(
            result.position.height.distance,
            expected_point.altitude_meters
        );
        assert_eq!(
            result.position.height.reference,
            RIDHeightReference::GroundLevel
        );
        assert_eq!(result.track, state.track_angle_degrees);
        assert_eq!(result.speed, state.ground_speed_mps);
        assert_eq!(result.speed_accuracy, SpeedAccuracy::SAUnknown);
        assert_eq!(result.vertical_speed, state.vertical_speed_mps);
    }

    #[test]
    fn test_from_recent_aircraft_position() {
        let expected_point = PointZ {
            latitude: rand::random(),
            longitude: rand::random(),
            altitude_meters: rand::random(),
        };

        let mut position = svc_gis_client_grpc::client::TimePosition {
            timestamp: Some(Utc::now().into()),
            position: Some(expected_point.clone()),
        };

        let _: RIDRecentAircraftPosition = position.clone().try_into().unwrap();

        position.timestamp = None;
        let error = TryInto::<RIDRecentAircraftPosition>::try_into(position.clone()).unwrap_err();
        assert_eq!(error, StatusCode::INTERNAL_SERVER_ERROR);

        let now = Utc::now();
        position.timestamp = Some(now.into());

        position.position = None;
        let error = TryInto::<RIDRecentAircraftPosition>::try_into(position.clone()).unwrap_err();
        assert_eq!(error, StatusCode::INTERNAL_SERVER_ERROR);
        position.position = Some(expected_point.clone());

        let result: RIDRecentAircraftPosition = position.clone().try_into().unwrap();
        assert_eq!(
            result.time.value,
            now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        );
        assert_eq!(result.position.lat, expected_point.latitude);
        assert_eq!(result.position.lng, expected_point.longitude);
        assert_eq!(result.position.alt, expected_point.altitude_meters);
        assert_eq!(result.position.accuracy_h, HorizontalAccuracy::HAUnknown);
        assert_eq!(result.position.accuracy_v, VerticalAccuracy::VAUnknown);
        assert_eq!(result.position.extrapolated, false);
        assert_eq!(result.position.pressure_alt, expected_point.altitude_meters);
        assert_eq!(
            result.position.height.distance,
            expected_point.altitude_meters
        );
        assert_eq!(
            result.position.height.reference,
            RIDHeightReference::GroundLevel
        );
    }

    #[test]
    fn test_from_flight() {
        let expected_point = PointZ {
            latitude: rand::random(),
            longitude: rand::random(),
            altitude_meters: rand::random(),
        };

        let mut flight = svc_gis_client_grpc::client::Flight {
            session_id: Some("session_id".to_string()),
            aircraft_id: Some("aircraft_id".to_string()),
            aircraft_type: 0,
            simulated: false,
            state: Some(svc_gis_client_grpc::client::AircraftState {
                timestamp: Some(Utc::now().into()),
                status: 0,
                position: Some(expected_point.clone()),
                track_angle_degrees: rand::random(),
                ground_speed_mps: rand::random(),
                vertical_speed_mps: rand::random(),
            }),
            positions: vec![svc_gis_client_grpc::client::TimePosition {
                timestamp: Some(Utc::now().into()),
                position: Some(expected_point.clone()),
            }],
        };

        let _: RIDFlight = flight.clone().try_into().unwrap();

        // IDs
        flight.session_id = None;
        flight.aircraft_id = None;
        let state = TryInto::<RIDFlight>::try_into(flight.clone()).unwrap();
        assert_eq!(state.id, "UNK".to_string());

        flight.aircraft_id = Some("aircraft_id".to_string());
        let state = TryInto::<RIDFlight>::try_into(flight.clone()).unwrap();
        assert_eq!(state.id, "aircraft_id".to_string());

        flight.session_id = Some("session_id".to_string());
        let state = TryInto::<RIDFlight>::try_into(flight.clone()).unwrap();
        assert_eq!(state.id, "session_id".to_string());

        flight.aircraft_type = 100;
        let state = TryInto::<RIDFlight>::try_into(flight.clone()).unwrap();
        assert_eq!(state.aircraft_type, UAType::NotDeclared);

        // invalid state
        flight.state = None;
        let error = TryInto::<RIDFlight>::try_into(flight.clone()).unwrap_err();
        assert_eq!(error, StatusCode::INTERNAL_SERVER_ERROR);
        flight.state = Some(svc_gis_client_grpc::client::AircraftState {
            timestamp: Some(Utc::now().into()),
            status: 0,
            position: Some(expected_point.clone()),
            track_angle_degrees: rand::random(),
            ground_speed_mps: rand::random(),
            vertical_speed_mps: rand::random(),
        });

        // flight.state.as_mut().unwrap().timestamp = None;
    }

    #[test]
    fn test_parse_coordinate() {
        assert_eq!(parse_coordinate("0.0", true).unwrap(), 0.0);
        assert_eq!(parse_coordinate("0.0", false).unwrap(), 0.0);
        assert_eq!(parse_coordinate("90.0", true).unwrap(), 90.0);
        assert_eq!(parse_coordinate("180.0", false).unwrap(), 180.0);

        let e = parse_coordinate("90.01", true).unwrap_err();
        assert_eq!(e, StatusCode::BAD_REQUEST);

        let e = parse_coordinate("180.01", false).unwrap_err();
        assert_eq!(e, StatusCode::BAD_REQUEST);

        // not a valid float
        let e = parse_coordinate("a", true).unwrap_err();
        assert_eq!(e, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_recent_flights() {
        let config = crate::config::Config::default();
        let grpc_clients = Extension(crate::grpc::client::GrpcClients::default(config));

        let window = Window {
            lon1: 4.850067,
            lat1: 52.392365,
            lon2: 4.906068,
            lat2: 52.371385,
        };

        // outside of allowable timeframe
        let e = get_recent_flights(&mut grpc_clients.clone(), &window, -0.0001)
            .await
            .unwrap_err();
        assert_eq!(e, StatusCode::BAD_REQUEST);

        // outside of allowable timeframe
        let e = get_recent_flights(&mut grpc_clients.clone(), &window, 60.0001)
            .await
            .unwrap_err();
        assert_eq!(e, StatusCode::BAD_REQUEST);

        // Valid request, but no lookback
        assert!(get_recent_flights(&mut grpc_clients.clone(), &window, 0.0)
            .await
            .unwrap()
            .is_empty());

        // Invalid window
        let window = Window {
            lon1: 4.850067,
            lat1: 52.392365,
            lon2: 4.959106,
            lat2: 90.0001,
        };
        let error = get_recent_flights(&mut grpc_clients.clone(), &window, 59.0)
            .await
            .unwrap_err();
        assert_eq!(error, StatusCode::BAD_REQUEST);

        let window = Window {
            lon1: 4.850067,
            lat1: 52.392365,
            lon2: 180.001,
            lat2: 52.364510,
        };
        let error = get_recent_flights(&mut grpc_clients.clone(), &window, 59.0)
            .await
            .unwrap_err();
        assert_eq!(error, StatusCode::BAD_REQUEST);

        // valid request
        let window = Window {
            lon1: 4.850067,
            lat1: 52.392365,
            lon2: 4.959106,
            lat2: 52.364510,
        };
        let _ = get_recent_flights(&mut grpc_clients.clone(), &window, 59.0)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_demo_flights() {
        let config = crate::config::Config::default();
        let grpc_clients = Extension(crate::grpc::client::GrpcClients::default(config));

        // valid window
        let request = GetFlightsRequest {
            view: "0.0,0.0,0.0,0.0".to_string(),
            recent_positions_duration: 0.0,
        };

        let _ = demo_flights(grpc_clients.clone(), Query(request))
            .await
            .unwrap();

        // Invalid window
        let request = GetFlightsRequest {
            view: "0.0,0.0,0.0".to_string(),
            recent_positions_duration: 0.0,
        };

        let e = demo_flights(grpc_clients.clone(), Query(request))
            .await
            .unwrap_err();
        assert_eq!(e, StatusCode::BAD_REQUEST);

        // Invalid window
        let request = GetFlightsRequest {
            view: "0.0,0.0,0.0,180.0001".to_string(),
            recent_positions_duration: 0.0,
        };

        let e = demo_flights(grpc_clients.clone(), Query(request))
            .await
            .unwrap_err();
        assert_eq!(e, StatusCode::BAD_REQUEST);

        let request = GetFlightsRequest {
            view: "0.0,0.0,90.001,0.0".to_string(),
            recent_positions_duration: 0.0,
        };
        let e = demo_flights(grpc_clients.clone(), Query(request))
            .await
            .unwrap_err();
        assert_eq!(e, StatusCode::BAD_REQUEST);
    }

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
            view: "0.0,0.0,0.0,0.0".to_string(),
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
