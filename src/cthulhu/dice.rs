use crate::context::Context;
use crate::error::{BotError, DiceRollingError};
use crate::parser::Amount;
use std::convert::TryFrom;
use std::fmt;

/// A planned dice roll.
#[derive(Clone, Debug, PartialEq)]
pub struct DiceRoll {
    pub amounts: Vec<Amount>,
    pub modifier: DiceRollModifier,
}

pub struct DiceRollWithContext<'a>(pub &'a DiceRoll, pub &'a Context<'a>);

/// Potential modifier on the die roll to be made.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DiceRollModifier {
    /// No bonuses or penalties.
    Normal,

    /// Roll one extra die and pick the lower of two results.
    OneBonus,

    /// Roll two extra dice and pick the lower of all results.
    TwoBonus,

    /// Roll one extra die and pick the higher of two results.
    OnePenalty,

    /// Roll two extra dice and pick the higher of all results.
    TwoPenalty,
}

impl fmt::Display for DiceRollModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Normal => "no modifiers",
            Self::OneBonus => "one bonus die",
            Self::TwoBonus => "two bonus dice",
            Self::OnePenalty => "one penalty die",
            Self::TwoPenalty => "two penalty dice",
        };

        write!(f, "{}", message)?;
        Ok(())
    }
}

/// The outcome of a die roll, either some kind of success or failure.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RollResult {
    /// Basic success. The rolled number was equal to or less than the target number.
    Success,

    /// Hard success means the rolled number was equal to or less than
    /// the target number divided by 2 (rounded down).
    HardSuccess,

    /// Extreme success means the rolled number was equal to or less
    /// than the target number divided by 5 (rounded down).
    ExtremeSuccess,

    /// A critical success occurs on a roll of 1.
    CriticalSuccess,

    /// A basic failure means that the roll was above the target number.
    Failure,

    /// A fumble occurs if the target number is below 50 and the roll
    /// was 96 - 100, OR if the roll result was 100. This means lower
    /// target numbers are more likely to produce a fumble.
    Fumble,
}

impl fmt::Display for RollResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Success => "success!",
            Self::HardSuccess => "hard success!",
            Self::ExtremeSuccess => "extreme success!",
            Self::CriticalSuccess => "critical success!",
            Self::Failure => "failure!",
            Self::Fumble => "fumble!",
        };

        write!(f, "{}", message)?;
        Ok(())
    }
}

pub struct ExecutedDiceRoll {
    /// The number we must meet for the roll to be considered a
    /// success.
    pub target: u32,

    /// Stored for informational purposes in display.
    pub modifier: DiceRollModifier,

    /// The actual roll result.
    pub roll: RolledDice,
}

impl fmt::Display for ExecutedDiceRoll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = format!("target: {}, with {}", self.target, self.modifier);
        write!(f, "{}", message)?;
        Ok(())
    }
}

//TODO need to keep track of all rolled numbers for informational purposes!
/// The outcome of a roll.
pub struct RolledDice {
    /// The d100 result actually rolled.
    num_rolled: u32,

    /// The number we must meet for the roll to be considered a
    /// success.
    target: u32,
}

impl RolledDice {
    /// Calculate what type of success or failure this roll is.
    /// Consult the RollResult enum for descriptions of what each
    /// result requires.
    pub fn result(&self) -> RollResult {
        let hard_target = self.target / 2u32;
        let extreme_target = self.target / 5u32;
        if (self.target < 50 && self.num_rolled > 95) || self.num_rolled == 100 {
            RollResult::Fumble
        } else if self.num_rolled == 1 {
            RollResult::CriticalSuccess
        } else if self.num_rolled <= extreme_target {
            RollResult::ExtremeSuccess
        } else if self.num_rolled <= hard_target {
            RollResult::HardSuccess
        } else if self.num_rolled <= self.target {
            RollResult::Success
        } else {
            RollResult::Failure
        }
    }
}

impl fmt::Display for RolledDice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = format!(
            "{} against {}: {}",
            self.num_rolled,
            self.target,
            self.result()
        );
        write!(f, "{}", message)?;
        Ok(())
    }
}

/// A planned advancement roll, where the target number is the
/// existing skill amount.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdvancementRoll {
    /// The amount (0 to 100) of the existing skill. We must beat this
    /// target number to advance the skill, or roll above a 95.
    pub existing_skill: u32,
}

impl fmt::Display for AdvancementRoll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = format!("advancement for skill of {}", self.existing_skill);
        write!(f, "{}", message)?;
        Ok(())
    }
}

/// A completed advancement roll.
pub struct RolledAdvancement {
    existing_skill: u32,
    num_rolled: u32,
    advancement: u32,
    successful: bool,
}

impl RolledAdvancement {
    /// The new skill amount, which will be the same if the roll was a
    /// failure.
    pub fn new_skill_amount(&self) -> u32 {
        self.existing_skill + self.advancement
    }

    /// How much the skill advanced (1 to 10). 0 if the advancement
    /// roll failed.
    pub fn advancement(&self) -> u32 {
        self.advancement
    }

    /// Whether or not the advancement roll was successful.
    pub fn successful(&self) -> bool {
        self.successful
    }
}

impl fmt::Display for RolledAdvancement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = if self.successful {
            format!(
                "success! new skill is {} (advanced by {}).",
                self.new_skill_amount(),
                self.advancement
            )
        } else {
            format!("failure! skill remains at {}", self.existing_skill)
        };

        write!(
            f,
            "rolled {} against {}: {}",
            self.num_rolled, self.existing_skill, message
        )?;
        Ok(())
    }
}

trait DieRoller {
    fn roll(&mut self) -> u32;
}

///A version of DieRoller that uses a rand::Rng to roll numbers.
struct RngDieRoller<R: rand::Rng>(R);

impl<R: rand::Rng> DieRoller for RngDieRoller<R> {
    fn roll(&mut self) -> u32 {
        self.0.gen_range(0, 10)
    }
}

/// Roll a single percentile die according to the rules. We cannot
/// simply roll a d100 due to the way the game calculates roll results
/// with bonus/penalty dice. The unit roll (ones place) is added to
/// the tens roll, unless both results are 0, in which case the result
/// is 100.
fn roll_percentile_dice<R: DieRoller>(roller: &mut R, unit_roll: u32) -> u32 {
    let tens_roll = roller.roll() * 10;

    if tens_roll == 0 && unit_roll == 0 {
        100
    } else {
        tens_roll + unit_roll
    }
}

fn roll_regular_dice<R: DieRoller>(
    modifier: &DiceRollModifier,
    target: u32,
    roller: &mut R,
) -> RolledDice {
    use DiceRollModifier::*;

    let num_rolls = match modifier {
        Normal => 1,
        OneBonus | OnePenalty => 2,
        TwoBonus | TwoPenalty => 3,
    };

    let unit_roll = roller.roll();

    let rolls: Vec<u32> = (0..num_rolls)
        .map(|_| roll_percentile_dice(roller, unit_roll))
        .collect();

    let num_rolled = match modifier {
        Normal => rolls.first(),
        OneBonus | TwoBonus => rolls.iter().min(),
        OnePenalty | TwoPenalty => rolls.iter().max(),
    }
    .unwrap();

    RolledDice {
        num_rolled: *num_rolled,
        target: target,
    }
}

fn roll_advancement_dice<R: DieRoller>(
    roll: &AdvancementRoll,
    roller: &mut R,
) -> RolledAdvancement {
    let unit_roll = roller.roll();
    let percentile_roll = roll_percentile_dice(roller, unit_roll);

    if percentile_roll > roll.existing_skill || percentile_roll > 95 {
        RolledAdvancement {
            num_rolled: percentile_roll,
            existing_skill: roll.existing_skill,
            advancement: roller.roll() + 1,
            successful: true,
        }
    } else {
        RolledAdvancement {
            num_rolled: percentile_roll,
            existing_skill: roll.existing_skill,
            advancement: 0,
            successful: false,
        }
    }
}

/// Make a roll with a target number and potential modifier. In a
/// normal roll, only one percentile die is rolled (1d100). With
/// bonuses or penalties, more dice are rolled, and either the lowest
/// (in case of bonus) or highest (in case of penalty) result is
/// picked. Rolls are not simply d100; the unit roll (ones place) is
/// rolled separately from the tens place, and then the unit number is
/// added to each potential roll before picking the lowest/highest
/// result.
pub async fn regular_roll(
    roll_with_ctx: &DiceRollWithContext<'_>,
) -> Result<ExecutedDiceRoll, BotError> {
    let target =
        crate::dice::calculate_dice_amount(&roll_with_ctx.0.amounts, roll_with_ctx.1).await?;

    let target = u32::try_from(target).map_err(|_| DiceRollingError::InvalidAmount)?;
    let mut roller = RngDieRoller(rand::thread_rng());
    let rolled_dice = roll_regular_dice(&roll_with_ctx.0.modifier, target, &mut roller);

    Ok(ExecutedDiceRoll {
        target: target,
        modifier: roll_with_ctx.0.modifier,
        roll: rolled_dice,
    })
}

impl AdvancementRoll {
    pub fn roll(&self) -> RolledAdvancement {
        let mut roller = RngDieRoller(rand::thread_rng());
        roll_advancement_dice(self, &mut roller)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::parser::{Amount, Element, Operator};

    /// Create a dummy room instance.
    fn dummy_room() -> matrix_sdk::Room {
        matrix_sdk::Room::new(
            &matrix_sdk::identifiers::room_id!("!fakeroomid:example.com"),
            &matrix_sdk::identifiers::user_id!("@fakeuserid:example.com"),
        )
    }

    /// Generate a series of numbers manually for testing. For this
    /// die system, the first roll in the Vec should be the unit roll,
    /// and any subsequent rolls should be the tens place roll. The
    /// results rolled must come from a d10 (0 to 9).
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
        fn roll(&mut self) -> u32 {
            let roll = self.results[self.position];
            self.position += 1;
            roll
        }
    }

    #[tokio::test]
    async fn regular_roll_converts_u32_safely() {
        let roll = DiceRoll {
            amounts: vec![Amount {
                operator: Operator::Plus,
                element: Element::Number(-10),
            }],
            modifier: DiceRollModifier::Normal,
        };

        let db = Database::new_temp().unwrap();
        let ctx = Context {
            db: db,
            matrix_client: &matrix_sdk::Client::new("https://example.com").unwrap(),
            room: &dummy_room(),
            username: "username",
            message_body: "message",
        };

        let roll_with_ctx = DiceRollWithContext(&roll, &ctx);
        let result = regular_roll(&roll_with_ctx).await;
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(BotError::DiceRollingError(DiceRollingError::InvalidAmount))
        ));
    }

    #[test]
    fn regular_roll_succeeds_when_below_target() {
        //Roll 30, succeeding.
        let mut roller = SequentialDieRoller::new(vec![0, 3]);
        let rolled = roll_regular_dice(&DiceRollModifier::Normal, 50, &mut roller);
        assert_eq!(RollResult::Success, rolled.result());
    }

    #[test]
    fn regular_roll_hard_success_when_rolling_half() {
        //Roll 25, succeeding.
        let mut roller = SequentialDieRoller::new(vec![5, 2]);
        let rolled = roll_regular_dice(&DiceRollModifier::Normal, 50, &mut roller);
        assert_eq!(RollResult::HardSuccess, rolled.result());
    }

    #[test]
    fn regular_roll_extreme_success_when_rolling_one_fifth() {
        //Roll 10, succeeding extremely.
        let mut roller = SequentialDieRoller::new(vec![0, 1]);
        let rolled = roll_regular_dice(&DiceRollModifier::Normal, 50, &mut roller);
        assert_eq!(RollResult::ExtremeSuccess, rolled.result());
    }

    #[test]
    fn regular_roll_extreme_success_target_above_100() {
        //Roll 30, succeeding extremely.
        let mut roller = SequentialDieRoller::new(vec![0, 3]);
        let rolled = roll_regular_dice(&DiceRollModifier::Normal, 150, &mut roller);
        assert_eq!(RollResult::ExtremeSuccess, rolled.result());
    }

    #[test]
    fn regular_roll_critical_success_on_one() {
        //Roll 1.
        let mut roller = SequentialDieRoller::new(vec![1, 0]);
        let rolled = roll_regular_dice(&DiceRollModifier::Normal, 50, &mut roller);
        assert_eq!(RollResult::CriticalSuccess, rolled.result());
    }

    #[test]
    fn regular_roll_fail_when_above_target() {
        //Roll 60.
        let mut roller = SequentialDieRoller::new(vec![0, 6]);
        let rolled = roll_regular_dice(&DiceRollModifier::Normal, 50, &mut roller);
        assert_eq!(RollResult::Failure, rolled.result());
    }

    #[test]
    fn regular_roll_is_fumble_when_skill_below_50_and_roll_at_least_96() {
        //Roll 96.
        let mut roller = SequentialDieRoller::new(vec![6, 9]);
        let rolled = roll_regular_dice(&DiceRollModifier::Normal, 49, &mut roller);
        assert_eq!(RollResult::Fumble, rolled.result());
    }

    #[test]
    fn regular_roll_is_failure_when_skill_at_or_above_50_and_roll_at_least_96() {
        //Roll 96.
        let mut roller = SequentialDieRoller::new(vec![6, 9]);
        let rolled = roll_regular_dice(&DiceRollModifier::Normal, 50, &mut roller);
        assert_eq!(RollResult::Failure, rolled.result());

        //Roll 96.
        let mut roller = SequentialDieRoller::new(vec![6, 9]);
        let rolled = roll_regular_dice(&DiceRollModifier::Normal, 68, &mut roller);
        assert_eq!(RollResult::Failure, rolled.result());
    }

    #[test]
    fn regular_roll_always_fumble_on_100() {
        //Roll 100.
        let mut roller = SequentialDieRoller::new(vec![0, 0]);
        let rolled = roll_regular_dice(&DiceRollModifier::Normal, 100, &mut roller);
        assert_eq!(RollResult::Fumble, rolled.result());
    }

    #[test]
    fn one_penalty_picks_highest_of_two() {
        //Should only roll 30 and 40, not 50.
        let mut roller = SequentialDieRoller::new(vec![0, 3, 4, 5]);
        let rolled = roll_regular_dice(&DiceRollModifier::OnePenalty, 50, &mut roller);
        assert_eq!(40, rolled.num_rolled);
    }

    #[test]
    fn two_penalty_picks_highest_of_three() {
        //Should only roll 30, 40, 50, and not 60.
        let mut roller = SequentialDieRoller::new(vec![0, 3, 4, 5, 6]);
        let rolled = roll_regular_dice(&DiceRollModifier::TwoPenalty, 50, &mut roller);
        assert_eq!(50, rolled.num_rolled);
    }

    #[test]
    fn one_bonus_picks_lowest_of_two() {
        //Should only roll 30 and 40, not 20.
        let mut roller = SequentialDieRoller::new(vec![0, 3, 4, 2]);
        let rolled = roll_regular_dice(&DiceRollModifier::OneBonus, 50, &mut roller);
        assert_eq!(30, rolled.num_rolled);
    }

    #[test]
    fn two_bonus_picks_lowest_of_three() {
        //Should only roll 30, 40, 50, and not 20.
        let mut roller = SequentialDieRoller::new(vec![0, 3, 4, 5, 2]);
        let rolled = roll_regular_dice(&DiceRollModifier::TwoBonus, 50, &mut roller);
        assert_eq!(30, rolled.num_rolled);
    }

    #[test]
    fn normal_modifier_rolls_once() {
        //Should only roll 30, not 40.
        let mut roller = SequentialDieRoller::new(vec![0, 3, 4]);
        let rolled = roll_regular_dice(&DiceRollModifier::Normal, 50, &mut roller);
        assert_eq!(30, rolled.num_rolled);
    }

    #[test]
    fn advancement_succeeds_on_above_skill() {
        let roll = AdvancementRoll { existing_skill: 30 };

        //Roll 52, then advance skill by 5. (advancement adds +1 to 0-9 roll)
        let mut roller = SequentialDieRoller::new(vec![2, 5, 4]);
        let rolled = roll_advancement_dice(&roll, &mut roller);
        assert!(rolled.successful());
        assert_eq!(5, rolled.advancement());
        assert_eq!(35, rolled.new_skill_amount());
    }

    #[test]
    fn advancement_succeeds_on_above_95() {
        let roll = AdvancementRoll { existing_skill: 97 };

        //Roll 96, then advance skill by 1. (advancement adds +1 to 0-9 roll)
        let mut roller = SequentialDieRoller::new(vec![6, 9, 0]);
        let rolled = roll_advancement_dice(&roll, &mut roller);
        assert!(rolled.successful());
        assert_eq!(1, rolled.advancement());
        assert_eq!(98, rolled.new_skill_amount());
    }

    #[test]
    fn advancement_fails_on_below_skill() {
        let roll = AdvancementRoll { existing_skill: 30 };

        //Roll 25, failing.
        let mut roller = SequentialDieRoller::new(vec![5, 2]);
        let rolled = roll_advancement_dice(&roll, &mut roller);
        assert!(!rolled.successful());
        assert_eq!(0, rolled.advancement());
    }
}
