use gpx::Gpx;
use serde::Deserialize;

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

// Public Structs
// Only parts relevant for the interpolation are here
/// Gps trackpoint representation used in lofi for Gps data
#[derive(Clone, Copy, Debug)]
pub struct GpsPoint {
    pub timestamp: i64,
    pub lat: f64,
    pub lon: f64,
    pub elevation: Option<f64>,
    pub accuracy: Option<f64>,
    pub vertical_accuracy: Option<f64>,
    pub bearing: Option<f64>,
    pub speed: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct Gps(HashMap<i64, GpsPoint>);

// Private Structs
// REEEeEeEeE format from our initial tracks
#[derive(Deserialize, Debug, Clone)]
struct GpsJson {
    time: i64,
    location: Location,
}

#[derive(Deserialize, Debug, Clone)]
struct Location {
    latitude: f64,
    longitude: f64,
    altitude: f64,
    accuracy: f64,
    vertical_accuracy: f64,
    bearing: f64,
    speed: f64,
    #[serde(rename = "elapsedMs")]
    _elapsed_ms: i32,
    _provider: String,
}

impl Gps {
    pub fn insert_from_legacy(&mut self, filepath: &str) {
        let file = File::open(filepath).expect("Could not open legacy json file");
        let rdr = BufReader::new(file);
        let points: Vec<GpsJson> =
            serde_json::from_reader(rdr).expect("Could not deserialize json");
        for p in points {
            self.insert(
                p.time,
                GpsPoint {
                    timestamp: p.time,
                    lat: p.location.latitude,
                    lon: p.location.longitude,
                    elevation: Some(p.location.altitude),
                    accuracy: Some(p.location.accuracy),
                    vertical_accuracy: Some(p.location.vertical_accuracy),
                    bearing: Some(p.location.bearing),
                    speed: Some(p.location.speed),
                },
            );
        }
    }

    /// Extracts waypoints from all tracks and segments of a Gpx file
    pub fn insert_from_gpx_file(&mut self, filepath: &str) {
        let file = File::open(filepath).expect("Could not open gpx file.");
        let reader = BufReader::new(file);
        let gpx = gpx::read(reader).expect("could not parse gpx");
        Gps::insert_from_gpx(self, gpx)
    }

    // GPX WayPoint & soul extractor
    /// Gets Gpx type object and extracts all Waypoints from it, Returns gps::Gps.
    fn insert_from_gpx(&mut self, gpx: Gpx) {
        // I feel like my IQ dropping around here, but dunno how to do it, especially given time
        // situation in gpx crate
        for track in gpx.tracks {
            for segment in track.segments {
                for point in segment.points {
                    let soul = GpsPoint {
                        lat: point.point().y(), // according to gpx crate team x and y are less
                        lon: point.point().x(), // ambiguous for coordinates on a map
                        elevation: point.elevation,
                        timestamp: match point.time {
                            Some(time) => chrono::naive::NaiveDateTime::parse_from_str(
                                &time.format().unwrap(),
                                "%Y-%m-%dT%H:%M:%SZ",
                            )
                            .unwrap()
                            .timestamp(),
                            None => break,
                        },

                        accuracy: point.pdop,
                        vertical_accuracy: point.vdop,
                        bearing: None,
                        speed: point.speed,
                    };

                    self.insert(soul.timestamp, soul);
                }
            }
        }
    }

    // hashmap boilerplate
    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = (&i64, &GpsPoint)> {
        self.0.iter()
    }

    #[allow(dead_code)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&i64, &mut GpsPoint)> {
        self.0.iter_mut()
    }

    pub fn insert(&mut self, k: i64, v: GpsPoint) -> Option<GpsPoint> {
        self.0.insert(k, v)
    }

    pub fn get(&self, k: &i64) -> Option<&GpsPoint> {
        self.0.get(k)
    }

    pub fn empty() -> Gps {
        Gps(HashMap::new())
    }
}
