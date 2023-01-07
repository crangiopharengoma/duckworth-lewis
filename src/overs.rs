use std::borrow::Borrow;
use std::ops::Sub;
use std::str::FromStr;

#[cfg(feature = "ser")]
use serde::{Deserialize, Serialize};

use crate::DuckworthLewisError;

/// A struct that represents a length of overs. Can handle both whole number of overs
/// and an incomplete number of overs
///
/// Implements `Sub`, `FromStr` and `TryFrom<f32>`. If a float with more than 1 decimal point
/// is used (e.g. 37.37) this will truncate it and return 37.3. No constructor is exposed
/// for partial overs, so using one of the conversion methdos is required.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "ser", derive(Serialize, Deserialize))]
pub struct Overs {
    pub overs: u16,
    balls: u16,
}

impl Overs {
    /// Constructor for overs; defaults balls to 0, so intended to be used only when describing
    /// a whole number of overs
    pub fn new(overs: u16) -> Overs {
        Overs { overs, balls: 0 }
    }

    /// The total number of balls that this length of overs contains
    pub fn total_balls(&self) -> u16 {
        self.overs * 6 + self.balls
    }
}

impl From<u16> for Overs {
    fn from(value: u16) -> Self {
        Overs::new(value)
    }
}

impl TryFrom<f32> for Overs {
    type Error = DuckworthLewisError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        let mut str = value.to_string();
        if let Some(point_ix) = str.find('.') {
            str.truncate(point_ix + 2);
            Ok(Overs::from_str(&str)?)
        } else if value < u16::MAX as f32 {
            Ok(Overs::new(value as u16))
        } else {
            Err(DuckworthLewisError::InvalidOverFormat(str))
        }
    }
}

impl FromStr for Overs {
    type Err = DuckworthLewisError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(num) = s.parse() {
            Ok(Overs::new(num))
        } else {
            let parts: Vec<_> = s.split('.').collect();
            if parts.len() != 2 {
                return Err(DuckworthLewisError::InvalidOverFormat(s.to_owned()));
            }
            let overs = parts[0].parse()?;
            let balls = parts[1].parse()?;

            if balls > 5 {
                return Err(DuckworthLewisError::TooManyBalls(balls));
            }

            Ok(Overs { overs, balls })
        }
    }
}

impl Sub<Overs> for &Overs {
    type Output = Overs;

    fn sub(self, rhs: Overs) -> Self::Output {
        subtract(self, &rhs)
    }
}

impl Sub for &Overs {
    type Output = Overs;

    fn sub(self, rhs: Self) -> Self::Output {
        subtract(self, rhs)
    }
}

impl Sub for Overs {
    type Output = Overs;

    fn sub(self, rhs: Self) -> Self::Output {
        subtract(&self, &rhs)
    }
}

impl Sub<&Overs> for Overs {
    type Output = Overs;

    fn sub(self, rhs: &Overs) -> Self::Output {
        subtract(&self, rhs)
    }
}

fn subtract<T: Borrow<Overs>>(lhs: &T, rhs: &T) -> Overs {
    let new_balls = lhs
        .borrow()
        .total_balls()
        .saturating_sub(rhs.borrow().total_balls());
    let (overs, balls) = if new_balls == 0 {
        (0, 0)
    } else {
        (new_balls / 6, new_balls % 6)
    };
    Overs { overs, balls }
}

#[cfg(test)]
mod test {
    use crate::overs::Overs;

    #[test]
    fn subtracting_exact_overs() {
        let overs_one = Overs::new(45);
        let overs_two = Overs::new(40);

        assert_eq!(overs_one - overs_two, Overs::new(5));
    }

    #[test]
    fn subtracting_partial_overs_without_overflow() {
        let overs_one = Overs {
            overs: 25,
            balls: 5,
        };
        let overs_two = Overs { overs: 5, balls: 2 };

        assert_eq!(
            overs_one - overs_two,
            Overs {
                overs: 20,
                balls: 3
            }
        );
    }

    #[test]
    fn subtracting_partial_overs_with_overflow() {
        let overs_one = Overs {
            overs: 25,
            balls: 2,
        };
        let overs_two = Overs { overs: 5, balls: 4 };

        assert_eq!(
            overs_one - overs_two,
            Overs {
                overs: 19,
                balls: 4
            }
        );
    }

    #[test]
    fn create_overs_from_str() {
        let o1: Overs = "37.3".parse().unwrap();
        let o2: Overs = "7.1".parse().unwrap();
        let o3: Overs = "50".parse().unwrap();
        let o4: Overs = "11.0".parse().unwrap();

        assert_eq!(
            o1,
            Overs {
                overs: 37,
                balls: 3
            }
        );
        assert_eq!(o2, Overs { overs: 7, balls: 1 });
        assert_eq!(
            o3,
            Overs {
                overs: 50,
                balls: 0
            }
        );
        assert_eq!(
            o4,
            Overs {
                overs: 11,
                balls: 0
            }
        );
    }

    #[test]
    #[should_panic]
    fn does_not_create_overs_from_invalid_float() {
        let _: Overs = 37.6.try_into().unwrap();
    }

    #[test]
    fn create_overs_from_float() {
        let o1: Overs = 37.3.try_into().unwrap();
        let o2: Overs = 7.1.try_into().unwrap();
        let o3: Overs = 50.0.try_into().unwrap();
        let o4: Overs = 11.32.try_into().unwrap();
        assert_eq!(
            o1,
            Overs {
                overs: 37,
                balls: 3
            }
        );
        assert_eq!(o2, Overs { overs: 7, balls: 1 });
        assert_eq!(
            o3,
            Overs {
                overs: 50,
                balls: 0
            }
        );
        assert_eq!(
            o4,
            Overs {
                overs: 11,
                balls: 3
            }
        );
    }
}
