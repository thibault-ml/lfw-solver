
extern crate serde;
extern crate serde_json;
use self::serde_json::Error as JsonError;
use self::serde_json::Value as JsonValue;
use self::serde_json::from_value as parse_json_value;


use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    InvalidJSONValue(JsonError),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidJSONValue(ref error) => error.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::InvalidJSONValue(ref error) => Some(error),
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidJSONValue(ref error) => {
                write!(fmt, "Unable to decode JSON object: {}", error)
            }
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub enum Location {
    Unknown,
    NeverVisitedAsOf(u32),
    VisitedAfter(u32),
    Visited,
}


#[derive(Debug, Serialize, Deserialize)]
pub enum MoveType {
    Regular,
    Alleyway,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Night {
    murder_location: u32,
    jack_nb_moves: u32,
    jack_known_locations: Vec<u32>,
    jack_alleyways: Vec<u32>,
    failed_clues_per_turn: Vec<Vec<usize>>,
    #[serde(skip_serializing, default)]
    locations: Vec<Location>,
}


impl Night {
    pub fn from_json(json: JsonValue) -> Result<Night, Error> {
        let mut night = try!(parse_json_value::<Night>(json.clone()).or_else(|e| {
            Err(Error::InvalidJSONValue(e))
        }));

        let mut locations = Vec::<Location>::with_capacity(195);
        for _ in 0..195 {
            locations.push(Location::Unknown);
        }

        for (zero_idx_turn, failed_clues_for_turn) in (&night.failed_clues_per_turn).iter().enumerate() {
            let turn = zero_idx_turn + 1; // We don't want turns to be zero indexed
            for failed_clue in failed_clues_for_turn {
                locations[failed_clue - 1] = Location::NeverVisitedAsOf(turn as u32)
            }
        }
        for location in &(night.jack_known_locations) {
            let zero_idx_loc = location - 1; // Night locations are not 0-indexed, but our array is
            locations[zero_idx_loc as usize] = match locations[zero_idx_loc as usize] {
                // There's no reason for a location to already be ::VisitedAfter, but never know...
                Location::NeverVisitedAsOf(turn) | Location::VisitedAfter(turn) => {
                    Location::VisitedAfter(turn)
                },
                _ => Location::Visited,
            };
        }

        night.locations = locations;

        Ok(night)
    }

    pub fn murder_location(&self) -> &u32 {
        &(self.murder_location)
    }

    pub fn jack_nb_moves(&self) -> &u32 {
        &(self.jack_nb_moves)
    }

    pub fn jack_alleyways(&self) -> &Vec<u32> {
        &(self.jack_alleyways)
    }

    pub fn jack_known_locations(&self) -> &Vec<u32> {
        &(self.jack_known_locations)
    }

    pub fn jack_visit_status_for_location(&self, location_index: &u32) -> &Location {
        &(self.locations[(location_index - 1) as usize])
    }

    pub fn jack_move_type_for_turn(&self, turn: &u32) -> MoveType {
        if self.jack_alleyways().contains(&(turn - 1)) {
            MoveType::Alleyway
        } else {
            MoveType::Regular
        }
    }
}
