
use lfw::Graph;
use lfw::Night;
use lfw::night::Location::*;
use lfw::night::MoveType;
use std::collections::VecDeque;


#[derive(Debug)]
struct LocationState {
    current_turn: u32,
    current_path: Vec<u32>,
}


#[derive(Debug)]
pub struct Solver {
    graph: Graph
}

impl Solver {
    pub fn new(graph: Graph) -> Solver {
        Solver {
            graph: graph
        }
    }

    pub fn precompute_shortest_distances(&self) -> Vec<Vec<u32>> {
        return vec![];
    }

    /**
     * Possible optimisations:
     * - Return if the number of moves left is less than the number of known locations we have yet to visit
     * When solving multiple nights:
     * - If we have a pre-calculated dijkstra distance, for night 2 onwards, we have a list of
     *   potential hideouts, exit early if there's no way to get to any of those in the number
     *   of moves left
     */
    pub fn solve_night(&self, night: Night) -> Option<Vec<Vec<u32>>> {
        let mut possible_paths = Vec::<Vec<u32>>::new();
        let mut current_path = Vec::<u32>::new();

        current_path.push(*night.murder_location());

        self.find_next_possible_locations(&night, &mut possible_paths, &mut current_path, &1);

        if possible_paths.is_empty() {
            return None;
        }
        Some(possible_paths)
    }

    fn find_next_possible_locations(&self,
        night: &Night,
        possible_paths: &mut Vec<Vec<u32>>,
        current_path: &mut Vec<u32>,
        current_turn: &u32)
    {
        if current_turn > night.jack_nb_moves() {
            // Check that the path we're testing includes all known locations for Jack
            if night.jack_known_locations().iter().all(|known_loc| current_path.contains(known_loc)) {
                possible_paths.push(current_path.clone());
            }
            return
        }

        // unwrap() because current_path shouldn't be empty, so we're happy to panic if that's the case
        let last_pos = *(current_path.last().unwrap());

        let connections = match night.jack_move_type_for_turn(current_turn) {
            MoveType::Alleyway => self.graph.alleyway_connections_for_location(&(last_pos)),
            MoveType::Regular => self.graph.connections_for_location(&(last_pos)),
        }.unwrap();

        for connection in connections {
            let should_visit = match *night.jack_visit_status_for_location(connection) {
                NeverVisitedAsOf(ref turn) | VisitedAfter(ref turn) if current_turn <= turn => false,
                _ => true
            };
            if should_visit {
                current_path.push(*connection);
                self.find_next_possible_locations(night, possible_paths, current_path, &(current_turn + 1));
                current_path.pop();
            }
        }
    }

    pub fn solve_night_non_recurs_slow(&self, night: Night) -> Option<Vec<Vec<u32>>> {
        let mut possible_paths = Vec::<Vec<u32>>::new();
        let mut queue: VecDeque<LocationState> = VecDeque::new();
        let max_nb_moves = *night.jack_nb_moves();

        let start_state = LocationState {
            current_turn: 1,
            current_path: vec![*night.murder_location()],
        };
        queue.push_back(start_state);

        while let Some(state) = queue.pop_front() {
            if state.current_turn > max_nb_moves {
                if night.jack_known_locations().iter().all(|known_loc| state.current_path.contains(known_loc)) {
                    possible_paths.push(state.current_path.clone());
                }
                continue;
            }

            // unwrap() because `current_path` shouldn't be empty, so we're happy to panic if that's the case
            let current_location = state.current_path.last().unwrap();

            let connections = match night.jack_move_type_for_turn(&(state.current_turn)) {
                MoveType::Alleyway => self.graph.alleyway_connections_for_location(current_location),
                MoveType::Regular => self.graph.connections_for_location(current_location),
            }.unwrap();

            for connection in connections {
                let should_visit = match *night.jack_visit_status_for_location(connection) {
                    NeverVisitedAsOf(ref turn) | VisitedAfter(ref turn) if state.current_turn <= *turn => false,
                    _ => true
                };
                if should_visit {
                    let mut current_path = state.current_path.clone();
                    current_path.push(*connection);
                    let new_state = LocationState {
                        current_turn: state.current_turn + 1,
                        current_path: current_path,
                    };
                    queue.push_back(new_state);
                }
            }
        }

        if possible_paths.is_empty() {
            return None;
        }
        Some(possible_paths)
    }
}
