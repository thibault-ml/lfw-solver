
use lfw::Graph;
use lfw::Night;
use lfw::night::Location::*;
use lfw::night::MoveType;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::iter::FromIterator;


#[derive(Debug)]
struct LocationState {
    current_turn: u32,
    current_path: Vec<u32>,
}


#[derive(Debug)]
pub struct Solver {
    graph: Graph,
    shortest_distances: Vec<Vec<u32>>,
}

impl Solver {
    pub fn from_graph(graph: Graph) -> Solver {
        let shortest_distances = Solver::precompute_shortest_distances(&graph);
        Solver {
            graph: graph,
            shortest_distances: shortest_distances
        }
    }

    // This method uses the Floyd–Warshall algorithm to precompute the shortest path between
    // all locations (graph nodes/vertices).
    #[allow(unknown_lints)]
    #[allow(needless_range_loop)]
    pub fn precompute_shortest_distances(graph: &Graph) -> Vec<Vec<u32>> {
        // Create a bidemensional table, 1000 value.
        let graph_size = 195;
        let mut dist: Vec<Vec<u32>> = vec![vec![1000; graph_size]; graph_size];

        // Initialises the graph
        // Set distance (v,v) = 0 on all locations (v [vertices])
        // Set distance (u,v) = (v,u) = 1 on all locations (u) and their connections (v)
        for location in 0..graph_size {
            dist[location][location] = 0;

            if let Some(all_cnx) = graph.all_connections_for_location(&(location as u32 + 1)) {
                for cnx in all_cnx {
                    let connection = *cnx as usize - 1;
                    dist[location][connection] = 1;
                    dist[connection][location] = 1;
                }
            }
        }

        // Iterate over intermediate locations
        for k in 0..graph_size {
            // Iterate over starting locations
            for i in 0..graph_size {
                // Iterate over end locations
                for j in 0..graph_size {
                    // If distance i->k + k->j is smaller that currently known i->j,
                    // then keep that distance as the minimum
                    let dist_ikj = dist[i][k] + dist[k][j];
                    if dist[i][j] > dist_ikj {
                        dist[i][j] = dist_ikj;
                        dist[j][i] = dist_ikj;
                    }
                }
            }
        }

        dist
    }

    fn distance_between_locations(&self, location: &u32, destination: &u32) -> u32 {
        self.shortest_distances[*location as usize - 1][*destination as usize - 1]
    }

    /**
     * TODO: Solve multiple nights. Now that we have a pre-calculated distance between each
     * location, we can further optimise searched by adding potential hideouts to the list of
     * locations we're require to have visited.
     */
    pub fn solve_night(&self, night: Night) -> Option<Vec<Vec<u32>>> {
        let mut possible_paths = Vec::<Vec<u32>>::new();
        let mut current_path = Vec::<u32>::new();
        let mut required_locations = HashSet::from_iter(night.jack_known_locations().iter());

        current_path.push(*night.murder_location());

        self.find_next_possible_locations(&night,
                                          &mut possible_paths,
                                          &mut current_path,
                                          &mut required_locations,
                                          &1);

        if possible_paths.is_empty() {
            return None;
        }
        Some(possible_paths)
    }

    /// Recursively finds Jack's next possible locations
    ///
    /// @param night The night to solve locations for
    /// @param possible_paths The list of paths we've determined were possible
    /// @param current_path The current patht to append onto
    /// @param required_locations The list of locations that still need to be visited
    /// @param current_turn The current turn/move being looked at
    fn find_next_possible_locations(&self,
        night: &Night,
        possible_paths: &mut Vec<Vec<u32>>,
        current_path: &mut Vec<u32>,
        required_locations: &mut HashSet<&u32>,
        current_turn: &u32)
    {
        if current_turn > night.jack_nb_moves() {
            // Check that the path we're testing includes all required locations for Jack.
            // It being empty means there are no required locations left to be visited.
            if required_locations.is_empty() {
                possible_paths.push(current_path.clone());
            }
            return
        }

        // unwrap() because current_path shouldn't be empty, so we're happy to panic if that's the case
        let current_location = *(current_path.last().unwrap());
        let moves_left = night.jack_nb_moves() - current_turn;

        // Simple but big optimisation here: if there's no way to go from the current location to
        // any of the required locations, then drop the current path and do not continue further.
        for required_location in required_locations.iter() {
            if self.distance_between_locations(&current_location, required_location) > moves_left {
                return
            }
        }

        let connections = match night.jack_move_type_for_turn(current_turn) {
            MoveType::Alleyway => self.graph.alleyway_connections_for_location(&(current_location)),
            MoveType::Regular => self.graph.connections_for_location(&(current_location)),
        }.unwrap();

        for connection in connections {
            let should_visit = match *night.jack_visit_status_for_location(connection) {
                NeverVisitedAsOf(ref turn) | VisitedAfter(ref turn) if current_turn <= turn => false,
                _ => true
            };
            if should_visit {
                current_path.push(*connection);
                let removed_required_location = if required_locations.contains(&current_location) {
                    required_locations.take(&current_location)
                } else {
                    None
                };
                self.find_next_possible_locations(night,
                                                  possible_paths,
                                                  current_path,
                                                  required_locations,
                                                  &(current_turn + 1));
                current_path.pop();
                if let Some(known_location) = removed_required_location {
                    required_locations.insert(known_location);
                }
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
