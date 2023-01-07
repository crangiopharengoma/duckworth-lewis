#[cfg(feature = "cli")]
use clap::ValueEnum;
use std::cmp::Ordering;

#[cfg(feature = "ser")]
use serde::{Deserialize, Serialize};

use crate::overs::Overs;
use crate::table::DUCKWORTH_LEWIS_TABLE;

const G50_FULL: f32 = 245.0;
const G50_OTHER: f32 = 200.0;

/// The relevant Grade of the teams involved in this match. Note that this is the teams grade not
/// the match grade. That means that the the grade is the highest level that that team is eligible
/// to play, not that the grade of the match being played.
///
/// For example, an ICC Full Member playing a warm up game against an invitational XI should still
/// have a grade of ICC Full Member for the purposes of calculating the match grade
///
/// Changing the grade determines the 'G50' value - i.e. the total runs expected in an 'average'
/// innings. The `CricketMatch` struct exposes a new_with_g_50 method that allows for some
/// experimentation if desired.
///
/// Current ICC playing conditions only has two G50 values - one for ICC Full Member and First Class
/// teams (245), one for all others (200).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ser", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum Grade {
    ICCFullMember,
    FirstClass,
    U19International,
    U15International,
    WomensInternational,
    ICCAssociateMember,
}

/// The innings that the match is in. As Duckworth Lewis is only relevant for limited overs matches
/// (and Cricket Max is dead), this can never include a third or a fourth innings
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "ser", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum Innings {
    First,
    Second,
}

/// Representation of a cricket match, used to hold the basic details of the match and the details
/// of any interruptions that have occurred during the match
#[cfg_attr(feature = "ser", derive(Serialize, Deserialize))]
pub struct CricketMatch {
    length: Overs,
    g_50: f32,
    interruptions: Vec<Interruption>,
}

#[derive(Debug)]
#[cfg_attr(feature = "ser", derive(Serialize, Deserialize))]
struct Interruption {
    wickets: u16,
    overs_left: Overs,
    overs_lost: Overs,
    innings: Innings,
}

impl CricketMatch {
    /// Create a new cricket match at a specified grade
    ///
    /// Grade affects the G50 value used to represent expected totals. Per standard ICC playing
    /// conditions, this is currently 245 for internationals between full ICC members and teams
    /// that play first class matches and 200 for all other matches
    ///
    /// Note that since this crate only supports Duckworth-Lewis Standard Edition the results
    /// will not match those seen in an official international match, which use the Duckworth-Lewis-Stern
    /// methodology (for which tables/calculations are not publicly available)
    pub fn new(length: Overs, grade: Grade) -> CricketMatch {
        assert!(length.overs <= 50);
        let g_50 = match grade {
            Grade::ICCFullMember | Grade::FirstClass => G50_FULL,
            _ => G50_OTHER,
        };
        let interruptions = Vec::new();
        CricketMatch {
            length,
            g_50,
            interruptions,
        }
    }

    /// Create a new cricket match with a custom G50 value
    ///
    /// Note that ICC has standard G50 values so this method shouldn't be used much, but is
    /// provided in case there is some reason to do so
    pub fn new_with_g_50(length: Overs, g_50: u16) -> CricketMatch {
        assert!(length.overs <= 50);
        let g_50 = g_50 as f32;
        let interruptions = Vec::new();
        CricketMatch {
            length,
            g_50,
            interruptions,
        }
    }

    /// Record an interruption has occurred. Wickets are total wickets lost in the innings,
    /// runs are total runs scored in the innings. Overs left are as at the beginning of
    /// the stoppage (i.e. not factoring in any adjustment because of this stoppage, but
    /// allowing for previous stoppages). Overs lost are the overs lost for this innings
    /// (i.e. if 20 overs total are lost, split as 10 overs per innings, over_lost = 10)
    ///
    /// Panics
    /// Wickets must less than 10
    /// Overs left must be less than or equal to the total length of the innings (e.g. 50)
    ///
    pub fn interruption(
        &mut self,
        wickets: u16,
        overs_left: Overs,
        overs_lost: Overs,
        innings: Innings,
    ) {
        assert!(wickets < 10);
        assert!(overs_left <= self.length);
        self.interruptions.push(Interruption {
            wickets,
            overs_left,
            overs_lost,
            innings,
        });
    }

    /// Returns the current target that the team batting second needs to have achieved
    /// at the conclusion of their innings. Assumes that innings 1 has been fully
    /// completed and all interruptions have been entered. If no interruptions
    /// have been entered, this will return 0
    ///
    /// If match has been abandoned after the second innings has completed the required
    /// minimum, this should be entered as an interruption and then revised_total will
    /// return the target total that the team batting second should have achieved at the
    /// point the match was abandoned in order to have won the match.
    ///
    /// This method does not alter internal state, so it can be called many times, including
    /// as a recalculation of the required total if additional interruptions occurred.
    ///
    /// This is a Mark Boucher friendly total - i.e. we're calculating the target not the par
    /// score
    pub fn revised_target(&self, first_innings_total: usize) -> u32 {
        if self.interruptions.is_empty() {
            return 0;
        }

        let (t1_resources, total_overs) = self
            .interruptions
            .iter()
            .filter(|int| int.innings == Innings::First)
            .fold(
                (self.initial_resources(), self.length.clone()),
                |(resources, overs), int| {
                    (resources - int.resource_loss(), overs - &int.overs_lost)
                },
            );

        let t2_resources = DUCKWORTH_LEWIS_TABLE.resources_remaining(&total_overs, 0);
        let t2_resources = self
            .interruptions
            .iter()
            .filter(|int| int.innings == Innings::Second)
            .fold(t2_resources, |resources, int| {
                resources - int.resource_loss()
            });

        let revised_target = match t2_resources.total_cmp(&t1_resources) {
            Ordering::Less => first_innings_total as f32 * (t2_resources / t1_resources) + 1.0,
            Ordering::Greater => {
                first_innings_total as f32 + (t2_resources - t1_resources) * self.g_50 / 100.0 + 1.0
            }
            Ordering::Equal => first_innings_total as f32,
        };

        revised_target as u32
    }

    /// Calculates the total resources available at the beginning of an innings
    fn initial_resources(&self) -> f32 {
        DUCKWORTH_LEWIS_TABLE.resources_remaining(&self.length, 0)
    }
}

impl Interruption {
    fn resource_loss(&self) -> f32 {
        let remaining_at_suspension =
            DUCKWORTH_LEWIS_TABLE.resources_remaining(&self.overs_left, self.wickets);
        let remaining_at_resumption = DUCKWORTH_LEWIS_TABLE
            .resources_remaining(&(&self.overs_left - &self.overs_lost), self.wickets);
        remaining_at_suspension - remaining_at_resumption
    }
}

#[cfg(test)]
mod test {
    use crate::game::{CricketMatch, Grade, Innings};
    use crate::Overs;

    /// Based on ICC example found here: https://icc-static-files.s3.amazonaws.com/ICC/document/2017/01/09/ca50a5e9-0241-494a-8773-d0cec059b31f/DuckworthLewis-Methodology.pdf
    #[test]
    fn icc_example_one() {
        let mut game = CricketMatch::new(Overs::new(50), Grade::ICCFullMember);
        game.interruption(3, Overs::new(30), Overs::new(10), Innings::First);
        let revised_total = game.revised_target(180);

        assert_eq!(revised_total, 185);
    }

    /// Based on ICC example found here: https://icc-static-files.s3.amazonaws.com/ICC/document/2017/01/09/ca50a5e9-0241-494a-8773-d0cec059b31f/DuckworthLewis-Methodology.pdf
    #[test]
    fn icc_example_two() {
        let mut game = CricketMatch::new(Overs::new(45), Grade::ICCFullMember);
        game.interruption(0, Overs::new(45), Overs::new(10), Innings::Second);
        let revised_total = game.revised_target(212);

        assert_eq!(revised_total, 185)
    }

    /// Based on ICC example found here: https://icc-static-files.s3.amazonaws.com/ICC/document/2017/01/09/ca50a5e9-0241-494a-8773-d0cec059b31f/DuckworthLewis-Methodology.pdf
    #[test]
    fn icc_example_three() {
        let mut game = CricketMatch::new(Overs::new(50), Grade::ICCFullMember);
        game.interruption(1, Overs::new(38), Overs::new(10), Innings::Second);
        let revised_total = game.revised_target(250);

        assert_eq!(revised_total, 218)
    }

    /// Based on ICC example found here: https://icc-static-files.s3.amazonaws.com/ICC/document/2017/01/09/ca50a5e9-0241-494a-8773-d0cec059b31f/DuckworthLewis-Methodology.pdf
    ///
    /// Note that this lib doesn't provide an API to calculate the par score. Therefore the target
    /// in this case (160) is different from the par score provided in the ICC example (159) above
    #[test]
    fn icc_example_four() {
        let mut game = CricketMatch::new(Overs::new(50), Grade::ICCFullMember);
        game.interruption(1, Overs::new(38), Overs::new(10), Innings::Second);
        game.interruption(3, Overs::new(18), Overs::new(2), Innings::Second);
        game.interruption(
            6,
            7.4.try_into().unwrap(),
            7.4.try_into().unwrap(),
            Innings::Second,
        );
        let revised_total = game.revised_target(250);
        assert_eq!(revised_total, 160)
    }
}
