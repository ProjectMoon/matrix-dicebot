use crate::roll::{Roll, Rolled};
use itertools::Itertools;
use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DicePoolQuality {
    TenAgain,
    NineAgain,
    EightAgain,
    Rote,
    ChanceDie,
    NoExplode,
}

impl fmt::Display for DicePoolQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DicePoolQuality::TenAgain => write!(f, "ten-again"),
            DicePoolQuality::NineAgain => write!(f, "nine-again"),
            DicePoolQuality::EightAgain => write!(f, "eight-again"),
            DicePoolQuality::Rote => write!(f, "rote quality"),
            DicePoolQuality::ChanceDie => write!(f, "chance die"),
            DicePoolQuality::NoExplode => write!(f, "no roll-agains"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DicePool {
    pub(crate) count: u32,
    pub(crate) sides: u32,
    pub(crate) success_on: u32,
    pub(crate) exceptional_success: u32,
    pub(crate) quality: DicePoolQuality,
}

impl fmt::Display for DicePool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} dice ({}, exceptional on {} successes)",
            self.count, self.quality, self.exceptional_success
        )
    }
}

impl DicePool {
    pub fn new(count: u32, successes_for_exceptional: u32, quality: DicePoolQuality) -> DicePool {
        DicePool {
            count: count,
            sides: 10, //TODO make configurable
            //TODO make configurable
            success_on: match quality {
                DicePoolQuality::ChanceDie => 10,
                _ => 8,
            },
            exceptional_success: successes_for_exceptional,
            quality: quality,
        }
    }

    pub fn chance_die() -> DicePool {
        DicePool {
            count: 1,
            sides: 10,
            success_on: 10,
            exceptional_success: 5,
            quality: DicePoolQuality::ChanceDie,
        }
    }
}

///Store all rolls of the dice pool dice into one struct.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DicePoolRoll {
    quality: DicePoolQuality,
    success_on: u32,
    exceptional_on: u32,
    rolls: Vec<u32>,
}

fn fmt_rolls(pool: &DicePoolRoll) -> String {
    let max_displayed_rolls = 15;
    let rolls = pool.rolls();
    if rolls.len() > max_displayed_rolls {
        let first_ten = rolls.iter().take(max_displayed_rolls).join(", ");
        format!(
            "{}, and {} more",
            first_ten,
            rolls.len() - max_displayed_rolls
        )
    } else {
        rolls.iter().take(10).join(", ")
    }
}

fn fmt_for_failure(pool: &DicePoolRoll) -> String {
    match pool.quality {
        //There should only be 1 die in a chance die roll.
        DicePoolQuality::ChanceDie if pool.rolls().first() == Some(&1) => {
            String::from("dramatic failure!")
        }
        _ => String::from("failure!"),
    }
}

impl DicePoolRoll {
    pub fn rolls(&self) -> &[u32] {
        &self.rolls
    }

    pub fn successes(&self) -> i32 {
        let successes = self
            .rolls
            .iter()
            .cloned()
            .filter(|&roll| roll >= self.success_on)
            .count();
        i32::try_from(successes).unwrap_or(0)
    }

    pub fn is_exceptional(&self) -> bool {
        self.successes() >= (self.exceptional_on as i32)
    }
}

impl Roll for DicePool {
    type Output = DicePoolRoll;

    fn roll(&self) -> DicePoolRoll {
        roll_dice(self, &mut RngDieRoller(rand::thread_rng()))
    }
}

impl Rolled for DicePoolRoll {
    fn rolled_value(&self) -> i32 {
        self.successes()
    }
}

impl fmt::Display for DicePoolRoll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let successes = self.successes();
        if successes > 0 {
            let success_msg = if self.is_exceptional() {
                format!("{} successes (exceptional!)", successes)
            } else {
                format!("{} successes", successes)
            };

            write!(f, "{} ({})", success_msg, fmt_rolls(&self))?;
        } else {
            write!(f, "{} ({})", fmt_for_failure(&self), fmt_rolls(&self))?;
        }

        Ok(())
    }
}

trait DieRoller {
    fn roll_number(&mut self, sides: u32) -> u32;
}

///A version of DieRoller that uses a rand::Rng to roll numbers.
struct RngDieRoller<R: rand::Rng>(R);

impl<R: rand::Rng> DieRoller for RngDieRoller<R> {
    fn roll_number(&mut self, sides: u32) -> u32 {
        self.0.gen_range(1, sides + 1)
    }
}

///Roll a die in the pool, that "explodes" on a given number or higher. Dice will keep
///being rolled until the result is lower than the explode number, which is normally 10.
///Statistically speaking, usually one result will be returned from this function.
fn roll_exploding_die<R: DieRoller>(
    roller: &mut R,
    sides: u32,
    explode_on_or_higher: u32,
) -> Vec<u32> {
    let mut results = vec![];
    loop {
        let roll = roller.roll_number(sides);
        results.push(roll);
        if roll < explode_on_or_higher {
            break;
        }
    }
    results
}

///A die with the rote quality is re-rolled once if the roll fails. Otherwise, it obeys
///all normal rules (re-roll 10s). Re-rolled dice are appended to the result set, so we
///can keep track of the actual dice that were rolled.
fn roll_rote_die<R: DieRoller>(roller: &mut R, sides: u32, success_on: u32) -> Vec<u32> {
    let mut rolls = roll_exploding_die(roller, sides, 10);

    if rolls.len() == 1 && rolls[0] < success_on {
        rolls.append(&mut roll_exploding_die(roller, sides, 10));
    }

    rolls
}

///Roll a single die in the pool, potentially rolling additional dice depending on  pool
///behavior. The default ten-again will "explode" the die if the result is 10 (repeatedly, if
///there are multiple 10s). Nine- and eight-again will explode similarly if the result is
///at least that number. Rote quality will re-roll a failure once, while also exploding
///on 10. The function returns a Vec of all rolled dice (usually 1).
fn roll_die<R: DieRoller>(roller: &mut R, pool: &DicePool) -> Vec<u32> {
    let mut results = vec![];
    let sides = pool.sides;

    match pool.quality {
        DicePoolQuality::TenAgain => results.append(&mut roll_exploding_die(roller, sides, 10)),
        DicePoolQuality::NineAgain => results.append(&mut roll_exploding_die(roller, sides, 9)),
        DicePoolQuality::EightAgain => results.append(&mut roll_exploding_die(roller, sides, 8)),
        DicePoolQuality::Rote => results.append(&mut roll_rote_die(roller, sides, pool.success_on)),
        DicePoolQuality::ChanceDie | DicePoolQuality::NoExplode => {
            results.push(roller.roll_number(sides))
        }
    }

    results
}

///Roll the dice in a dice pool, according to behavior documented in the various rolling
///methods.
fn roll_dice<R: DieRoller>(pool: &DicePool, roller: &mut R) -> DicePoolRoll {
    let rolls: Vec<u32> = (0..pool.count)
        .flat_map(|_| roll_die(roller, pool))
        .collect();

    DicePoolRoll {
        quality: pool.quality,
        rolls: rolls,
        exceptional_on: pool.exceptional_success,
        success_on: pool.success_on,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    ///Instead of being random, generate a series of numbers we have complete
    ///control over.
    struct SequentialDieRoller {
        results: Vec<u32>,
        position: usize,
    }

    impl SequentialDieRoller {
        fn new(results: Vec<u32>) -> SequentialDieRoller {
            SequentialDieRoller {
                results: results,
                position: 0,
            }
        }
    }

    impl DieRoller for SequentialDieRoller {
        fn roll_number(&mut self, _sides: u32) -> u32 {
            let roll = self.results[self.position];
            self.position += 1;
            roll
        }
    }

    //Sanity checks
    #[test]
    pub fn chance_die_has_success_on_10_test() {
        assert_eq!(
            10,
            DicePool::new(1, 5, DicePoolQuality::ChanceDie).success_on
        );
    }

    #[test]
    pub fn non_chance_die_has_success_on_8_test() {
        fn check_success_on(quality: DicePoolQuality) {
            assert_eq!(8, DicePool::new(1, 5, quality).success_on);
        }

        check_success_on(DicePoolQuality::TenAgain);
        check_success_on(DicePoolQuality::NineAgain);
        check_success_on(DicePoolQuality::EightAgain);
        check_success_on(DicePoolQuality::Rote);
        check_success_on(DicePoolQuality::NoExplode);
    }

    //Dice rolling tests.
    #[test]
    pub fn ten_again_test() {
        let mut roller = SequentialDieRoller::new(vec![10, 8, 1]);
        let rolls = roll_exploding_die(&mut roller, 10, 10);
        assert_eq!(vec![10, 8], rolls);
    }

    #[test]
    pub fn nine_again_test() {
        let mut roller = SequentialDieRoller::new(vec![10, 9, 8, 1]);
        let rolls = roll_exploding_die(&mut roller, 10, 9);
        assert_eq!(vec![10, 9, 8], rolls);
    }

    #[test]
    pub fn eight_again_test() {
        let mut roller = SequentialDieRoller::new(vec![10, 9, 8, 8, 1]);
        let rolls = roll_exploding_die(&mut roller, 10, 8);
        assert_eq!(vec![10, 9, 8, 8, 1], rolls);
    }

    #[test]
    pub fn rote_quality_fail_then_succeed_test() {
        let mut roller = SequentialDieRoller::new(vec![5, 8, 1]);
        let rolls = roll_rote_die(&mut roller, 10, 8);
        assert_eq!(vec![5, 8], rolls);
    }

    #[test]
    pub fn rote_quality_fail_twice_test() {
        let mut roller = SequentialDieRoller::new(vec![5, 6, 10]);
        let rolls = roll_rote_die(&mut roller, 10, 8);
        assert_eq!(vec![5, 6], rolls);
    }

    #[test]
    pub fn rote_quality_fail_then_explode_test() {
        let mut roller = SequentialDieRoller::new(vec![5, 10, 8, 1]);
        let rolls = roll_rote_die(&mut roller, 10, 8);
        assert_eq!(vec![5, 10, 8], rolls);
    }

    #[test]
    pub fn rote_quality_obeys_success_on_test() {
        //With success_on = 8, should only roll once.
        let mut roller = SequentialDieRoller::new(vec![8, 7]);
        let rolls = roll_rote_die(&mut roller, 10, 8);
        assert_eq!(vec![8], rolls);

        //With success_on = 9, we should re-roll if it's an 8.
        roller = SequentialDieRoller::new(vec![8, 7]);
        let rolls = roll_rote_die(&mut roller, 10, 9);
        assert_eq!(vec![8, 7], rolls);
    }

    #[test]
    pub fn no_explode_roll_test() {
        let pool = DicePool::new(1, 5, DicePoolQuality::NoExplode);
        let mut roller = SequentialDieRoller::new(vec![10, 8]);
        let roll: DicePoolRoll = roll_dice(&pool, &mut roller);
        assert_eq!(vec![10], roll.rolls());
    }

    //DicePool tests
    #[test]
    fn is_successful_on_equal_test() {
        let result = DicePoolRoll {
            quality: DicePoolQuality::TenAgain,
            rolls: vec![8],
            exceptional_on: 5,
            success_on: 8,
        };

        assert_eq!(1, result.successes());
    }

    #[test]
    fn chance_die_success_test() {
        let result = DicePoolRoll {
            quality: DicePoolQuality::TenAgain,
            rolls: vec![10],
            exceptional_on: 5,
            success_on: 10,
        };

        assert_eq!(1, result.successes());
    }

    #[test]
    fn chance_die_fail_test() {
        let result = DicePoolRoll {
            quality: DicePoolQuality::TenAgain,
            rolls: vec![9],
            exceptional_on: 5,
            success_on: 10,
        };

        assert_eq!(0, result.successes());
    }

    #[test]
    fn is_exceptional_test() {
        let result = DicePoolRoll {
            quality: DicePoolQuality::TenAgain,
            rolls: vec![8, 8, 9, 10, 8],
            exceptional_on: 5,
            success_on: 8,
        };

        assert_eq!(5, result.successes());
        assert_eq!(true, result.is_exceptional());
    }

    #[test]
    fn is_not_exceptional_test() {
        let result = DicePoolRoll {
            quality: DicePoolQuality::TenAgain,
            rolls: vec![8, 8, 9, 10],
            exceptional_on: 5,
            success_on: 8,
        };

        assert_eq!(4, result.successes());
        assert_eq!(false, result.is_exceptional());
    }

    //Format tests
    #[test]
    fn formats_dramatic_failure_test() {
        let result = DicePoolRoll {
            quality: DicePoolQuality::ChanceDie,
            rolls: vec![1],
            exceptional_on: 5,
            success_on: 10,
        };

        assert_eq!("dramatic failure!", fmt_for_failure(&result));
    }

    #[test]
    fn formats_regular_failure_when_not_chance_die_test() {
        let result = DicePoolRoll {
            quality: DicePoolQuality::TenAgain,
            rolls: vec![1],
            exceptional_on: 5,
            success_on: 10,
        };

        assert_eq!("failure!", fmt_for_failure(&result));
    }

    #[test]
    fn formats_lots_of_dice_test() {
        let result = DicePoolRoll {
            quality: DicePoolQuality::TenAgain,
            rolls: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            exceptional_on: 5,
            success_on: 10,
        };

        assert_eq!(
            "1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 1, 2, 3, 4, 5, and 4 more",
            fmt_rolls(&result)
        );
    }
}
