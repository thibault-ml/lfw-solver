
extern crate serde;
extern crate serde_json;
use self::serde::de;
use self::serde_json::Error as JsonError;
use self::serde_json::Value as JsonValue;
use self::serde_json::from_value as parse_json_value;


use std::error;
use std::fmt;
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum Error {
    GameDataNotAnObject,
    MissingKey(String),
    InvalidValueForKey(String, JsonError),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::GameDataNotAnObject => "invalid game data",
            Error::MissingKey(_) => "key not found",
            Error::InvalidValueForKey(_, ref error) => error.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::InvalidValueForKey(_, ref error) => Some(error),
            _ => None,
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::GameDataNotAnObject => {
                write!(fmt, "Game data is not a JSON Object")
            },
            Error::MissingKey(ref key) => {
                write!(fmt, "Could not find required game data key: {}", key)
            },
            Error::InvalidValueForKey(ref key, ref error) => {
                write!(fmt, "Invalid value for key {}: {}", key, error)
            }
        }
    }
}


#[derive(Debug)]
pub enum Location {
    Unknown,
    NeverVisitedAsOf(u32),
    VisitedAfter(u32),
    Visited,
}


#[derive(Debug)]
pub struct Game {
    murder_location: u32,
    jack_nb_moves: u32,
    jack_known_locations: Vec<u32>,
    jack_alleyways: Vec<u32>,
    locations: Vec<Location>,
}


impl Game {
    fn parse_key<T>(key: &str, game_object: &BTreeMap<String, JsonValue>) -> Result<T, Error>
        where T: de::Deserialize
    {
        let json_item = try!(game_object.get(key).ok_or(Error::MissingKey(key.to_string())));
        let value = try!(parse_json_value::<T>(json_item.clone()).or_else(|e| {
            Err(Error::InvalidValueForKey(key.to_string(), e))
        }));
        return Ok(value);
    }

    pub fn from_json(json: JsonValue) -> Result<Game, Error> {
        let game_object = try!(json.as_object().ok_or(Error::GameDataNotAnObject));

        let murder_location = try!(Game::parse_key::<u32>("murder_location", &game_object));
        let jack_known_locations = try!(Game::parse_key::<Vec<u32>>("jack_known_locations", &game_object));
        let jack_nb_moves = try!(Game::parse_key::<u32>("jack_nb_moves", &game_object));
        let jack_alleyways = try!(Game::parse_key::<Vec<u32>>("jack_alleyways", &game_object));
        let failed_clues = try!(Game::parse_key::<Vec<Vec<usize>>>("failed_clues_per_turn", &game_object));

        let mut locations = Vec::<Location>::with_capacity(195);
        for _ in 0..195 {
            locations.push(Location::Unknown);
        }
        for (zero_idx_turn, failed_clues_for_turn) in failed_clues.iter().enumerate() {
            let turn = zero_idx_turn + 1; // We don't want turns to be zero indexed
            for failed_clue in failed_clues_for_turn {
                locations[failed_clue - 1] = Location::NeverVisitedAsOf(turn as u32)
            }
        }
        for location in &jack_known_locations {
            let zero_idx_loc = location - 1; // Game location is not 0-indexed, but our array is
            locations[zero_idx_loc as usize] = match locations[zero_idx_loc as usize] {
                // There's no reason for a location to already be ::VisitedAfter, but never know...
                Location::NeverVisitedAsOf(turn) | Location::VisitedAfter(turn) => {
                    Location::VisitedAfter(turn)
                },
                _ => Location::Visited,
            };
        }

        return Ok(Game {
            murder_location: murder_location,
            jack_nb_moves: jack_nb_moves,
            jack_known_locations: jack_known_locations,
            jack_alleyways: jack_alleyways,
            locations: locations,
        });
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
}
