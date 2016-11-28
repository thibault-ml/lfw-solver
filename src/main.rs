
#[macro_use]
extern crate lfw_solver;
extern crate serde_json;
extern crate getopts;
use lfw_solver::*;
use getopts::Options;
use std::{env, process};
use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use serde_json::*;
use serde_json::Value as JsonValue;


fn print_usage(options: Options, program_executable: &str, print_small_usage: bool) {
    let short_usage = options.short_usage(program_executable);
    if print_small_usage {
        println!("{}", short_usage);
    } else {
        println!("{}", options.usage(&short_usage));
    }
}


fn parse_json_file_at_path<P: AsRef<Path>>(json_path: P) -> Result<JsonValue> {
    let graph_file = try!(File::open(json_path));
    let reader = BufReader::new(graph_file);
    serde_json::from_reader(reader)
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let program_executable = args[0].clone();

    let mut options = Options::new();
    options.optflag("h", "help", "Display this help message");
    options.optopt("g", "graph", "Path to a JSON file representing the LfW graph", "GRAPH");
    options.optopt("d",
                   "game-data",
                   "Path to a JSON file containing the game data",
                   "GAME_DATA");

    let matches = match options.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println_stderr!("{}", e);
            print_usage(options, &program_executable, true);
            process::exit(1);
        }
    };

    // Parse help
    if matches.opt_present("h") {
        print_usage(options, &program_executable, false);
        process::exit(1);
    }

    let graph_file_path = matches.opt_str("graph").unwrap_or("./lfw-graph.json".to_string());
    let json_graph_data = match parse_json_file_at_path(graph_file_path) {
        Ok(d) => d,
        Err(e) => {
            println_stderr!("Could not open graph file: {}", e);
            process::exit(1);
        }
    };

    let game_data_path = matches.opt_str("game-data").unwrap_or("./lfw-game-data.json".to_string());
    let json_night_data = match parse_json_file_at_path(game_data_path) {
        Ok(d) => d,
        Err(e) => {
            println_stderr!("Could not open game data file: {}", e);
            process::exit(1);
        }
    };

    let graph = match lfw::Graph::from_json(json_graph_data) {
        Ok(g) => g,
        Err(e) => {
            println_stderr!("Failed to load graph data: {}", e);
            process::exit(1);
        }
    };

    let night = match lfw::Night::from_json(json_night_data) {
        Ok(g) => g,
        Err(e) => {
            println_stderr!("Failed to load game data: {}", e);
            process::exit(1);
        }
    };
    // println!("{:?}", night);

    let solver = lfw::Solver::new(graph);

    match solver.solve_night(night) {
        Some(solutions) => {
            println!("Jack potentially used the following paths:");
            for solution in &solutions {
                println!("{:?}", solution);
            }
        }
        None => {
            println_stderr!("No solution found. Have you entered the data correctly?");
            process::exit(1);
        }
    }
}
