use std::fmt;

/// A planned dice roll.
#[derive(Clone, Copy)]
pub struct DiceRoll {
    pub target: u32,
    pub modifier: DiceRollModifier,
}

impl fmt::Display for DiceRoll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = format!("target: {}, modifiers: {}", self.target, self.modifier);
        write!(f, "{}", message)?;
        Ok(())
    }
}

/// Potential modifier on the die roll to be made.
#[derive(Clone, Copy)]
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
            Self::Normal => "none",
            Self::OneBonus => "one bonus",
            Self::TwoBonus => "two bonus",
            Self::OnePenalty => "one penalty",
            Self::TwoPenalty => "two penalty",
        };

        write!(f, "{}", message)?;
        Ok(())
    }
}

/// The outcome of a die roll, either some kind of success or failure.
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

/// The outcome of a roll.
pub struct RolledDice {
    /// The d100 result actually rolled.
    num_rolled: u32,

    /// The number we must meet for the roll to be considered a
    /// success.
    target: u32,

    /// Stored for informational purposes in display.
    modifier: DiceRollModifier,
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
pub struct AdvancementRoll {
    /// The amount (0 to 100) of the existing skill. We must beat this
    /// target number to advance the skill, or roll above a 95.
    pub existing_skill: u32,
}

/// A completed advancement roll.
pub struct RolledAdvancement {
    existing_skill: u32,
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

impl DiceRoll {
    /// Make a roll with a target number and potential modifier. In a
    /// normal roll, only one percentile die is rolled (1d100). With
    /// bonuses or penalties, more dice are rolled, and either the lowest
    /// (in case of bonus) or highest (in case of penalty) result is
    /// picked. Rolls are not simply d100; the unit roll (ones place) is
    /// rolled separately from the tens place, and then the unit number is
    /// added to each potential roll before picking the lowest/highest
    /// result.
    pub fn roll(&self) -> RolledDice {
        use DiceRollModifier::*;
        let num_rolls = match self.modifier {
            Normal => 1,
            OneBonus | OnePenalty => 2,
            TwoBonus | TwoPenalty => 3,
        };

        let mut roller = RngDieRoller(rand::thread_rng());
        let unit_roll = roller.roll();

        let rolls: Vec<u32> = (0..num_rolls)
            .map(|_| roll_percentile_dice(&mut roller, unit_roll))
            .collect();

        let num_rolled = match self.modifier {
            Normal => rolls.first(),
            OneBonus | TwoBonus => rolls.iter().min(),
            OnePenalty | TwoPenalty => rolls.iter().max(),
        }
        .unwrap();

        RolledDice {
            modifier: self.modifier,
            num_rolled: *num_rolled,
            target: self.target,
        }
    }
}

impl AdvancementRoll {
    pub fn roll(&self) -> RolledAdvancement {
        let mut roller = RngDieRoller(rand::thread_rng());
        let unit_roll = roller.roll();
        let percentile_roll = roll_percentile_dice(&mut roller, unit_roll);

        if percentile_roll < self.existing_skill || percentile_roll > 95 {
            RolledAdvancement {
                existing_skill: self.existing_skill,
                advancement: roller.roll() + 1,
                successful: true,
            }
        } else {
            RolledAdvancement {
                existing_skill: self.existing_skill,
                advancement: 0,
                successful: false,
            }
        }
    }
}
