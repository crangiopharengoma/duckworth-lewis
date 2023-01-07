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
