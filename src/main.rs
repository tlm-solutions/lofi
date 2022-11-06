mod filter;
mod gps;
mod stops;
mod structs;
mod crayon;

use structs::{Cli, Command, MergeArgs, StopsToGeoArgs};
use crate::filter::filter_cmd;

use dump_dvb::locations::LocationsJson;
use dump_dvb::telegrams::r09::R09SaveTelegram;

use std::fs::{write, File};

use clap::Parser;
use geojson::{Feature, FeatureCollection, Geometry, JsonObject, JsonValue, Value};
use crate::structs::CrayonArgs;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Correlate(opts) => stops::correlate_cmd(opts),
        Command::Merge(opts) => merge(opts),
        Command::StopsToGeo(opts) => stops2geo(opts),
        Command::Filter(opts) => filter_cmd(opts),
        Command::Crayon(opts) => invoke_crayon(opts)
    }
}

fn merge(_opts: MergeArgs) {
    todo!();
}

/// Convert the json-formatted locations to geojson, useful for debug
fn stops2geo(opts: StopsToGeoArgs) {
    let mut features: Vec<Feature> = vec![];
    for path in opts.stops {
        let stops = LocationsJson::from_file(&path).expect("Couldn't deserialize stops file");
        features.append(&mut get_features(&stops));
    }

    let feature_collection = FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    };
    let geojson_string = feature_collection.to_string();

    match opts.geojson {
        Some(path) => {
            write(path, geojson_string).expect("Couldn't write geojson");
        }
        None => {
            println!("{}", geojson_string);
        }
    }
}

fn read_telegrams(paths: Vec<String>) -> Box<dyn Iterator<Item = R09SaveTelegram>> {
    Box::new(paths
        .into_iter()
        .map(|p| File::open(p).expect("couldn't open file"))
        .map(csv::Reader::from_reader)
        .map(|r| r.into_deserialize())
        .flat_map(|tg| {
            // TODO proper result<Option<>, > handling
            tg.filter_map(|t| {
                match t {
                    Ok(data) => {
                        data
                    },
                    Err(e) => {
                        println!("error {:?}", e);
                        None
                    }
                }
            })
        }))
}

fn get_features(locs: &LocationsJson) -> Vec<Feature> {
    let mut features: Vec<Feature> = vec![];
    for (_reg, v) in locs.data.iter() {
        for (mp, loc) in v {
            let mut properties = JsonObject::new();
            let propval = format!("{}", mp);
            properties.insert("name".to_string(), JsonValue::from(propval));
            features.push(Feature {
                bbox: None,
                geometry: Some(Geometry::new(Value::Point(vec![loc.lon, loc.lat]))),
                id: None,
                properties: Some(properties),
                foreign_members: None,
            })
        }
    }

    features
}

fn invoke_crayon(args: CrayonArgs) {
    crayon::correlate_lines(args);
}
