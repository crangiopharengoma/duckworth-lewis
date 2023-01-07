//! This is a rust lib that allows for the calculation of target scores for the
//! team batting second in a cricket match that has been affected by weather
//! using the Duckworth Lewis Standard Edition method. Currently the Professional
//! Edition and Duckworth-Lewis-Stern methodologies aren't published (anywhere
//! that I'm aware of) so I can't implement them here. Note that international
//! cricket uses the Duckworth-Lewis-Stern method so the results from this lib
//! won't match what you see on TV.

use std::num::ParseIntError;

use thiserror::Error;

pub use game::{CricketMatch, Grade, Innings};
pub use overs::Overs;

mod game;
mod overs;
mod table;

#[derive(Error, Debug)]
pub enum DuckworthLewisError {
    #[error("overs must be in the format <overs>.<balls> got {0}")]
    InvalidOverFormat(String),
    #[error("balls must be less than 6, got {0}")]
    TooManyBalls(u16),
    #[error("{0}")]
    OversNotNumeric(String),
}

impl From<ParseIntError> for DuckworthLewisError {
    fn from(value: ParseIntError) -> Self {
        DuckworthLewisError::OversNotNumeric(value.to_string())
    }
}
