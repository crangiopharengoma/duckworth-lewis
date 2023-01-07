use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

use duckworth_lewis::{CricketMatch, Grade, Innings, Overs};

type Store = HashMap<usize, MatchWrapper>;

#[derive(Debug, Parser)]
#[command(name = "dlc")]
#[command(about = "A calculator for determining targets for the team batting second in weather affected cricket matches using the Duckworth Lewis Standard Edition methodology", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Specify where you want the calculator to store information about matches - can also be set by environment variable
    #[clap(short, long = "store")]
    #[arg(env = "DUCKWORTH_LEWIS_STORAGE", default_value = "store.json")]
    store_location: PathBuf,
    /// Specify the id of the match you are updating; if not provided defaults to latest match created
    #[clap(short, long)]
    id: Option<usize>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new match
    New {
        /// Number of overs available when the first ball is delivered
        length: Overs,
        /// Highest grade the teams playing in this match are eligible to play
        #[arg(value_enum)]
        grade: Grade,
        /// Name of team batting first
        #[clap(long = "team_1")]
        #[arg(default_value = "Team 1")]
        team_1: String,
        /// Name of team batting second
        #[clap(long = "team_2")]
        #[arg(default_value = "Team 2")]
        team_2: String,
    },
    /// Add an interruption to an existing match
    Int {
        /// Total wickets lost in the innings so far
        wickets: u16,
        /// Overs remaining in the innings at the time the innings was interrupted (i.e. before any deductions are made for this interruption)
        overs_left: Overs,
        /// Overs lost in this innings from this interruption (e.g. if 10 overs are lost, reducing each innings to 45 overs, then this should be 5)
        overs_lost: Overs,
        /// Which innings the interruption occurred during
        innings: Innings,
    },
    /// Calculate the current second innings target for an existing match; this can be safely recalculated if additional interruptions occur
    Target {
        /// Total runs scored in the first innings (not the par score)
        first_innings_total: usize,
    },
    /// List all matches that are currently held in the match store
    List,
    /// Delete matches from the store
    Delete {
        /// Match id to delete
        match_ids: Vec<usize>,
    },
}

/// A simple wrapper that allows saving matches with ids
#[derive(Serialize, Deserialize)]
struct MatchWrapper {
    match_id: usize,
    creation: Duration,
    game: CricketMatch,
    team_1: String,
    team_2: String,
}

impl MatchWrapper {
    fn new(
        match_id: usize,
        length: Overs,
        grade: Grade,
        team_1: String,
        team_2: String,
    ) -> MatchWrapper {
        MatchWrapper {
            match_id,
            game: CricketMatch::new(length, grade),
            creation: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time went backwards"),
            team_1,
            team_2,
        }
    }

    fn add_int(&mut self, wickets: u16, overs_left: Overs, overs_lost: Overs, innings: Innings) {
        self.game
            .interruption(wickets, overs_left, overs_lost, innings);
    }

    fn calc_target(&self, first_innings_total: usize) -> u32 {
        self.game.revised_target(first_innings_total)
    }
}

fn main() {
    let args: Cli = Cli::parse();
    let mut store = get_store(&args.store_location);

    match args.command {
        Commands::New {
            length,
            grade,
            team_1,
            team_2,
        } => {
            let id = store.keys().max().unwrap_or(&0) + 1;
            let game = MatchWrapper::new(id, length, grade, team_1, team_2);
            store.insert(id, game);
            persist_store(store, args.store_location);
        }
        Commands::Int {
            wickets,
            overs_left,
            overs_lost,
            innings,
        } => {
            retrieve_game(args.id, &mut store).add_int(wickets, overs_left, overs_lost, innings);
            persist_store(store, args.store_location);
        }
        Commands::Target {
            first_innings_total,
        } => println!(
            "Adjusted target for team 2 is {}",
            retrieve_game(args.id, &mut store).calc_target(first_innings_total)
        ),
        Commands::List => store.values().for_each(|game| {
            println!(
                "Match {} between {} and {}",
                game.match_id, game.team_1, game.team_2
            )
        }),
        Commands::Delete { match_ids } => {
            let set: HashSet<_> = match_ids.into_iter().collect();
            store.retain(|k, _| !set.contains(k));
            persist_store(store, args.store_location);
        }
    }
}

fn retrieve_game(id: Option<usize>, store: &mut Store) -> &mut MatchWrapper {
    match id {
        Some(id) => store
            .get_mut(&id)
            .unwrap_or_else(|| panic!("match with id {} not found", id)),
        None => store.get_mut(&get_latest(store)).unwrap(),
    }
}

fn get_latest(store: &Store) -> usize {
    store
        .values()
        .max_by_key(|value| value.creation)
        .expect("no matches created yet")
        .match_id
}

fn persist_store(store: Store, location: PathBuf) {
    let location =
        BufWriter::new(File::create(location).expect("failed to open match store for writing"));
    serde_json::to_writer(location, &store).expect("failed to save match details");
}

fn get_store(location: &PathBuf) -> Store {
    if let Ok(file) = File::open(location) {
        serde_json::from_reader(BufReader::new(file)).expect("failed to read match details")
    } else {
        Store::new()
    }
}
