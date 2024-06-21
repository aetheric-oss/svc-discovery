/// Types used for REST communication with the svc-cargo server

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use lib_common::time::{SecondsFormat, Utc};
use std::fmt::Debug;
use strum::{EnumString, Display, EnumIter};

/// RFC3339 format enum
pub const RFC3339_FORMAT_STRING: &str = "RFC3339";

/// Example Request Body Information Type
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GetFlightsRequest {
    /// The area of this view as a string of format "lat1,lon1,lat2,lon2"
    pub view: String,

    /// Recent positions duration
    pub recent_positions_duration: f32
}

/// A time in RFC3339 format
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct Time {
    /// RFC3339-formatted time/date string. The time zone must be 'Z'.
    /// example: '1985-04-12T23:20:50.52Z'
    pub value: String,

    /// The format of the time string RFC3339
    pub format: String
}

impl Default for Time {
    fn default() -> Time {
        Time {
            value: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
            format: RFC3339_FORMAT_STRING.to_string()
        }
    }
}

/// An altitude with variable reference and units
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct Altitude {
    /// The altitude in meters
    pub value: f64,

    /// A code indicating the reference for a vertical distance.
    /// Only supports WGS84 at present
    pub reference: String, // W84

    /// The units of the altitude (M)
    pub units: String, // M    
}

impl Default for Altitude {
    fn default() -> Altitude {
        Altitude {
            value: -1000.0, // unknown or invalid
            reference: "W84".to_string(),
            units: "M".to_string()
        }
    }

}

/// A point defined by a latitude and longitude
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct LatLngPoint {
    /// Degrees of longitude ([-180, 180])
    /// Invalid, no value, or unknown is 0
    pub lng: f64,

    /// Degrees of latitude ([-90, 90])
    /// Invalid, no value, or unknown is 0
    pub lat: f64
}

/// Defines a 2D area around a point
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct Radius {
    /// The magnitude of the radius
    pub value: f32,

    /// The units of the radius (only M supported)
    pub units: String
}

impl Default for Radius {
    fn default() -> Radius {
        Radius {
            value: 0.001,
            units: "M".to_string()
        }
    }
}

/// A circle defined by a center point (long, lat) and a radius
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct Circle {
    /// The center of the circle
    pub center: LatLngPoint,

    /// The radius of the circle
    pub radius: Radius
}

/// A polygon defined by a list of points (at least 3)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct Polygon {
    /// The points of the polygon
    pub vertices: Vec<LatLngPoint>
}

/// A volume defined by a circle or polygon and a lower and upper altitude
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct Volume3D {
    /// The outline of the volume as a circle
    outline_circle: Circle,

    /// The outline of the volume as a polygon
    outline_polygon: Polygon,

    /// The altitude of the lower bound of the volume
    altitude_lower: Altitude,

    /// The altitude of the upper bound of the volume
    altitude_upper: Altitude
}

/// A 4D volume defined by a 3D volume and a start and end time
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct Volume4D {
    /// The 3D volume
    pub volume: Volume3D,

    /// The start time of the volume
    pub time_start: Time,

    /// The end time of the volume
    pub time_end: Time
}

/// The state of a flight, including position and speed
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct RIDAircraftState {
    /// The time of the state
    pub timestamp: Time,

    /// The accuracy of the timestamp
    pub timestamp_accuracy: f32,

    /// The operational status of the aircraft
    pub operational_status: RIDOperationalStatus,

    /// The position of the aircraft
    pub position: RIDAircraftPosition,

    /// The track of the aircraft with respect to true north
    pub track: f32,

    /// The speed of the aircraft
    pub speed: f32,

    /// The accuracy of the speed
    pub speed_accuracy: SpeedAccuracy,

    /// The vertical speed of the aircraft
    pub vertical_speed: f32,
}

/// The operating area of a flight
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct OperatingArea {
    /// The number of aircraft in the area
    pub aircraft_count: i32,

    /// The volume of the operating area
    /// ASTM spec indicates this is a list of <OperatingArea>, not <Volume4D> - typo?
    pub volumes: Vec<Volume4D>
}

/// The height of an asset
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct RIDHeight {
    /// The height in meters
    pub distance: f32,

    /// The reference for the height
    pub reference: RIDHeightReference
}

/// The position of an aircraft
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct RIDAircraftPosition {
    /// Degrees of latitude ([-90, 90])
    /// Invalid, no value, or unknown is 0
    pub lat: f64,

    /// Degrees of longitude ([-180, 180])
    /// Invalid, no value, or unknown is 0
    pub lng: f64,

    /// geodetic altitude in meters
    /// invalid, no value, or unknown is -1000
    pub alt: f32,

    /// horizontal accuracy in meters
    pub accuracy_h: HorizontalAccuracy,

    /// vertical accuracy in meters
    pub accuracy_v: VerticalAccuracy,

    /// is extrapolated rather than reported
    pub extrapolated: bool,

    /// pressure altitude in meters
    /// invalid, no value, or unknown is -1000
    pub pressure_alt: f32,

    /// indicated altitude
    pub height: RIDHeight
}

/// The position of an aircraft at a reported time
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct RIDRecentAircraftPosition {
    /// Reported time of the position
    pub time: Time,

    /// The position of the aircraft
    pub position: RIDAircraftPosition
}

/// A flight in the area
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct RIDFlight {
    /// The ID identifies a flight, and is unique to a remote ID service provider who is providing remote ID services.
    pub id: String,

    /// The type of aircraft
    pub aircraft_type: UAType,

    /// The state of the aircraft
    pub current_state: RIDAircraftState,

    /// The operating area of the aircraft
    pub operating_area: OperatingArea,

    /// If this is a simulated flight, this will be true
    pub simulated: bool,

    /// The recent positions of the aircraft
    pub recent_positions: Vec<RIDRecentAircraftPosition>
}

/// The response to a get_flights request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GetFlightsResponse {
    /// The time of the response
    pub timestamp: Time,

    /// The flights in the area
    pub flights: Vec<RIDFlight>,

    /// If no ISAs are present, this will be true
    pub no_isas_present: bool
}

impl Default for GetFlightsResponse {
    fn default() -> GetFlightsResponse {
        GetFlightsResponse {
            timestamp: Time {
                value: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                format: RFC3339_FORMAT_STRING.to_string()
            },
            flights: Vec::new(),
            no_isas_present: false
        }
    }
}

/// The type of aircraft
#[derive(Debug, Display, Copy, Clone, EnumString, EnumIter, Serialize, Deserialize,PartialEq)]
#[derive(ToSchema)]
#[schema(default="NotDeclared")]
pub enum UAType {
    /// Not declared
    NotDeclared,

    /// Aeroplane
    Aeroplane,

    /// Helicopter
    Helicopter,

    /// Gyroplane
    Gyroplane,

    /// Hybrid lift
    HybridLift,

    /// Ornithopter
    Ornithopter,

    /// Glider
    Glider,

    /// Kite
    Kite,

    /// Free balloon
    FreeBalloon,

    /// Captive balloon
    CaptiveBalloon,

    /// Airship
    Airship,

    /// Free fall or parachute
    FreeFallOrParachute,

    /// Rocket
    Rocket,

    /// Tethered powered aircraft
    TetheredPoweredAircraft,

    /// Ground obstacle
    GroundObstacle,

    /// Other
    Other,
}

/// Vertical accuracy of position in meters
#[derive(Debug, Display, Copy, Clone, EnumString, EnumIter, Serialize, Deserialize, PartialEq)]
#[derive(ToSchema)]
#[schema(default="VAUnknown")]
pub enum VerticalAccuracy {
    /// Unknown
    VAUnknown,

    /// 150m or more
    VA150mPlus,

    /// < 150m
    VA150m,

    /// < 45m
    VA45m,

    /// < 25m
    VA25m,

    /// < 10m
    VA10m,

    /// < 3m
    VA3m,

    /// < 1m
    VA1m
}

/// Horizontal accuracy of position in meters
#[derive(Debug, Display, Copy, Clone, EnumString, EnumIter, Serialize, Deserialize, PartialEq)]
#[derive(ToSchema)]
#[schema(default="HAUnknown")]
pub enum HorizontalAccuracy {
    /// Unknown
    HAUnknown,

    /// 10NM (18.52km) or more
    HA10NMPlus,

    /// < 10NM (18.52km)
    HA10NM,

    /// < 4NM (7.408km)
    HA4NM,

    /// < 2NM (3.704km)
    HA2NM,

    /// < 1NM (1.852km)
    HA1NM,

    /// < 0.5NM (926m)
    HA05NM,

    /// < 0.3NM (555.6m)
    HA03NM,

    /// < 0.1NM (185.2m)
    HA01NM,

    /// < 0.05NM (92.6m)
    HA005NM,

    /// < 30m
    HA30m,

    /// < 10m
    HA10m,

    /// < 3m
    HA3m,

    /// < 1m
    HA1m,
}

/// The reference for the height
#[derive(Debug, Display, Copy, Clone, EnumString, EnumIter, Serialize, Deserialize, PartialEq)]
#[derive(ToSchema)]
pub enum RIDHeightReference {
    /// Takeoff location
    TakeoffLocation,

    /// Ground level
    GroundLevel
}

/// The reference for the vertical distance
#[derive(Debug, Display, Copy, Clone, EnumString, EnumIter, Serialize, Deserialize, PartialEq)]
#[derive(ToSchema)]
pub enum SpeedAccuracy {
    /// Unknown
    SAUnknown,

    /// 10m/s or more
    SA10mpsPlus,

    /// < 10m/s
    SA10mps,

    /// < 3m/s
    SA3mps,

    /// < 1m/s
    SA1mps,

    /// < 0.3m/s
    SA03mps,
}

/// The operational status of the aircraft
#[derive(Debug, Display, Copy, Clone, EnumString, EnumIter, Serialize, Deserialize, PartialEq)]
#[derive(ToSchema)]
#[schema(default="Undeclared")]
pub enum RIDOperationalStatus {
    /// Undeclared
    Undeclared,

    /// Ground
    Ground,

    /// Airborne
    Airborne,

    /// Emergency
    Emergency,

    /// Remote ID System Failure
    RemoteIDSystemFailure,
}
