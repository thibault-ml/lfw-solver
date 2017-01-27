
extern crate serde_json;
use self::serde_json::Error as JsonError;
use self::serde_json::Value as JsonValue;
use self::serde_json::from_value as parse_json_value;

use std::collections::BTreeMap;
use std::collections::HashSet;
use std::error;
use std::fmt;
use std::iter::FromIterator;
use std::ops::Index;


pub const MIN_INDEX: usize = 1;
pub const MAX_INDEX: usize = 195;
pub const REG_MOVE_KEY: &'static str = "regular_moves";
pub const ALLEY_MOVE_KEY: &'static str = "alleyway_moves";


#[derive(Debug)]
pub enum Error {
    GraphNotAnObject,
    LocationNotFound(u32),
    LocationNotParsable(u32, JsonError),
    MissingKey(String),
    InvalidLocationValue(u32),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::GraphNotAnObject => "invalid graph argument",
            Error::LocationNotFound(_) => "location not found",
            Error::LocationNotParsable(_, ref error) => error.description(),
            Error::MissingKey(_) => "missing key",
            Error::InvalidLocationValue(_) => "invalid location value",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::LocationNotParsable(_, ref error) => Some(error),
            _ => None,
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::GraphNotAnObject => {
                write!(fmt, "Graph argument is not a JSON Object")
            },
            Error::LocationNotFound(ref idx) => {
                write!(fmt, "Could not find expected location: {}", idx)
            },
            Error::LocationNotParsable(ref idx, ref error) => {
                write!(fmt, "Value for location {} not parsable: {}", idx, error)
            },
            Error::MissingKey(ref key) => {
                write!(fmt, "Could not find key {}", key)
            },
            Error::InvalidLocationValue(ref loc) => {
                write!(fmt, "Invalid location value {} (must be 1...195)", loc)
            },
        }
    }
}


#[derive(Debug)]
pub struct Graph {
    // locations: Vec<Vec<u32>>,
    regular_connections: Vec<Vec<u32>>,
    alleyway_connections: Vec<Vec<u32>>,
}


impl Graph {
    fn verified_connections(all_connections: &BTreeMap<String, Vec<u32>>, key: &str) -> Result<Vec<u32>, Error> {
        let connections = try!(all_connections
                                .get(key)
                                .ok_or(Error::MissingKey(key.to_string())));

        connections.iter().map(|cnx| {
            match *cnx {
                c @ 1 ... 195 => Ok(c),
                _ => Err(Error::InvalidLocationValue(*cnx))
            }
        }).collect::<Result<Vec<u32>, Error>>()
    }

    pub fn from_json(json: JsonValue) -> Result<Graph, Error> {
        // Not using `as_object` since it gives us a ref, and we'd have to clone the values.
        // We're not interested in the JSON object once we parse it, so it's fine to move values.
        let mut location_map = try!(match json {
            JsonValue::Object(obj) => Some(obj),
            _ => None
        }.ok_or(Error::GraphNotAnObject));

        let mut all_regular_connections: Vec<Vec<u32>> = Vec::new();
        let mut all_alleyway_connections: Vec<Vec<u32>> = Vec::new();

        for x in 0..195 {
            let idx = x + 1;
            let location = try!(location_map
                                .remove( &(idx.to_string()) )
                                .ok_or(Error::LocationNotFound(idx)));
            let all_connections = try!(parse_json_value::<BTreeMap<String, Vec<u32>>>(location).or_else(|e| {
                Err(Error::LocationNotParsable(idx, e))
            }));

            let regular_cnx = try!(Graph::verified_connections(&all_connections, REG_MOVE_KEY));
            all_regular_connections.push(regular_cnx);

            let alleyway_cnx = try!(Graph::verified_connections(&all_connections, ALLEY_MOVE_KEY));
            all_alleyway_connections.push(alleyway_cnx);
        }
        Ok(Graph {
            regular_connections: all_regular_connections,
            alleyway_connections: all_alleyway_connections,
        })
    }

    pub fn connections_for_location(&self, location: &u32) -> Option<&Vec<u32>> {
        if let loc @ 1 ... 195 = *location {
            self.regular_connections.get((loc as usize) - 1)
        } else {
            None
        }
    }

    pub fn alleyway_connections_for_location(&self, location: &u32) -> Option<&Vec<u32>> {
        if let loc @ 1 ... 195 = *location {
            self.alleyway_connections.get((loc as usize) - 1)
        } else {
            None
        }
    }

    pub fn all_connections_for_location(&self, location: &u32) -> Option<Vec<&u32>> {
        if let loc @ 1 ... 195 = *location {
            let mut all_cnx_set = HashSet::new();
            if let Some(regular_cnx) = self.regular_connections.get(loc as usize - 1) {
                for cnx in regular_cnx {
                    all_cnx_set.insert(cnx);
                }
            }
            if let Some(alleyway_cnx) = self.alleyway_connections.get(loc as usize - 1) {
                for cnx in alleyway_cnx {
                    all_cnx_set.insert(cnx);
                }
            }
            let all_cnx = Vec::from_iter(all_cnx_set.into_iter());
            return Some(all_cnx);
        }
        None
    }
}

impl Index<usize> for Graph {
    type Output = Vec<u32>;

    fn index(&self, location: usize) -> &Vec<u32> {
        match location {
            MIN_INDEX...MAX_INDEX => &(self.regular_connections[location - 1]),
            loc => {
                panic!("Invalid location {} (must be between {} and {})",
                        loc, MIN_INDEX, MAX_INDEX)
            }
        }
    }
}
