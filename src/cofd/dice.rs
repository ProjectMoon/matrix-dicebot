use crate::context::Context;
use crate::db::DataError::KeyDoesNotExist;
use crate::error::BotError;
use crate::roll::{Roll, Rolled};
use itertools::Itertools;
use std::convert::TryFrom;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiceRollingError {
    #[error("variable not found: {0}")]
    VariableNotFound(String),

    #[error("dice pool expression too large")]
    ExpressionTooLarge,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Operator {
    Plus,
    Minus,
}

impl Operator {
    pub fn mult(&self) -> i32 {
        match self {
            Operator::Plus => 1,
            Operator::Minus => -1,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Element {
    Variable(String),
    Number(i32),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Amount {
    pub operator: Operator,
    pub element: Element,
}

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct DicePoolModifiers {
    pub(crate) success_on: i32,
    pub(crate) exceptional_on: i32,
    pub(crate) quality: DicePoolQuality,
}

impl DicePoolModifiers {
    pub fn default() -> DicePoolModifiers {
        DicePoolModifiers {
            success_on: 8,
            exceptional_on: 5,
            quality: DicePoolQuality::TenAgain,
        }
    }

    pub fn custom_quality(quality: DicePoolQuality) -> DicePoolModifiers {
        let success_on = if quality != DicePoolQuality::ChanceDie {
            8
        } else {
            10
        };
        DicePoolModifiers {
            success_on: success_on,
            exceptional_on: 5,
            quality: quality,
        }
    }

    pub fn custom_exceptional_on(exceptional_on: i32) -> DicePoolModifiers {
        DicePoolModifiers {
            success_on: 8,
            exceptional_on: exceptional_on,
            quality: DicePoolQuality::TenAgain,
        }
    }

    pub fn custom(quality: DicePoolQuality, exceptional_on: i32) -> DicePoolModifiers {
        DicePoolModifiers {
            success_on: 8,
            exceptional_on: exceptional_on,
            quality: quality,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DicePool {
    pub(crate) amounts: Vec<Amount>,
    pub(crate) sides: i32,
    pub(crate) modifiers: DicePoolModifiers,
}

fn calculate_dice_amount(pool: &DicePoolWithContext) -> Result<i32, BotError> {
    let dice_amount: Result<i32, BotError> = pool
        .0
        .amounts
        .iter()
        .map(|amount| match &amount.element {
            Element::Number(num_dice) => Ok(*num_dice * amount.operator.mult()),
            Element::Variable(variable) => handle_variable(&pool.1, &variable),
        })
        .collect::<Result<Vec<i32>, _>>()
        .map(|numbers| numbers.iter().sum());

    dice_amount
}

impl DicePool {
    pub fn easy_pool(dice_amount: i32, quality: DicePoolQuality) -> DicePool {
        DicePool {
            amounts: vec![Amount {
                operator: Operator::Plus,
                element: Element::Number(dice_amount),
            }],
            sides: 10,
            modifiers: DicePoolModifiers::custom_quality(quality),
        }
    }

    pub fn easy_with_modifiers(dice_amount: i32, modifiers: DicePoolModifiers) -> DicePool {
        DicePool {
            amounts: vec![Amount {
                operator: Operator::Plus,
                element: Element::Number(dice_amount),
            }],
            sides: 10,
            modifiers: modifiers,
        }
    }

    pub fn new(amounts: Vec<Amount>, modifiers: DicePoolModifiers) -> DicePool {
        DicePool {
            amounts: amounts,
            sides: 10, //TODO make configurable
            //TODO make configurable
            modifiers: modifiers,
        }
    }

    pub fn chance_die() -> DicePool {
        DicePool::easy_pool(1, DicePoolQuality::ChanceDie)
    }
}

///The result of a successfully executed roll of a dice pool. Does not
///contain the heavy information of the DicePool instance.
pub struct RolledDicePool {
    pub(crate) num_dice: i32,
    pub(crate) roll: DicePoolRoll,
    pub(crate) modifiers: DicePoolModifiers,
}

impl RolledDicePool {
    fn from(pool: &DicePool, num_dice: i32, rolls: Vec<i32>) -> RolledDicePool {
        RolledDicePool {
            modifiers: pool.modifiers,
            num_dice: num_dice,
            roll: DicePoolRoll {
                rolls: rolls,
                modifiers: pool.modifiers,
            },
        }
    }
}

impl fmt::Display for RolledDicePool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} dice ({}, exceptional on {} successes)",
            self.num_dice, self.modifiers.quality, self.modifiers.exceptional_on
        )
    }
}

///Store all rolls of the dice pool dice into one struct.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DicePoolRoll {
    modifiers: DicePoolModifiers,
    rolls: Vec<i32>,
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
    match pool.modifiers.quality {
        //There should only be 1 die in a chance die roll.
        DicePoolQuality::ChanceDie if pool.rolls().first() == Some(&1) => {
            String::from("dramatic failure!")
        }
        _ => String::from("failure!"),
    }
}

impl DicePoolRoll {
    pub fn rolls(&self) -> &[i32] {
        &self.rolls
    }

    pub fn successes(&self) -> i32 {
        let successes = self
            .rolls
            .iter()
            .cloned()
            .filter(|&roll| roll >= self.modifiers.success_on)
            .count();
        i32::try_from(successes).unwrap_or(0)
    }

    pub fn is_exceptional(&self) -> bool {
        self.successes() >= self.modifiers.exceptional_on
    }
}

/// Attach a Context to a dice pool. Needed for database access.
pub struct DicePoolWithContext<'a>(pub &'a DicePool, pub &'a Context);

impl Roll for DicePoolWithContext<'_> {
    type Output = Result<RolledDicePool, BotError>;

    fn roll(&self) -> Result<RolledDicePool, BotError> {
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
    fn roll_number(&mut self, sides: i32) -> i32;
}

///A version of DieRoller that uses a rand::Rng to roll numbers.
struct RngDieRoller<R: rand::Rng>(R);

impl<R: rand::Rng> DieRoller for RngDieRoller<R> {
    fn roll_number(&mut self, sides: i32) -> i32 {
        self.0.gen_range(1, sides + 1)
    }
}

///Roll a die in the pool, that "explodes" on a given number or higher. Dice will keep
///being rolled until the result is lower than the explode number, which is normally 10.
///Statistically speaking, usually one result will be returned from this function.
fn roll_exploding_die<R: DieRoller>(
    roller: &mut R,
    sides: i32,
    explode_on_or_higher: i32,
) -> Vec<i32> {
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
fn roll_rote_die<R: DieRoller>(roller: &mut R, sides: i32, success_on: i32) -> Vec<i32> {
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
fn roll_die<R: DieRoller>(roller: &mut R, pool: &DicePool) -> Vec<i32> {
    let mut results = vec![];
    let sides = pool.sides;
    let success_on = pool.modifiers.success_on;

    match pool.modifiers.quality {
        DicePoolQuality::TenAgain => results.append(&mut roll_exploding_die(roller, sides, 10)),
        DicePoolQuality::NineAgain => results.append(&mut roll_exploding_die(roller, sides, 9)),
        DicePoolQuality::EightAgain => results.append(&mut roll_exploding_die(roller, sides, 8)),
        DicePoolQuality::Rote => results.append(&mut roll_rote_die(roller, sides, success_on)),
        DicePoolQuality::ChanceDie | DicePoolQuality::NoExplode => {
            results.push(roller.roll_number(sides))
        }
    }

    results
}

fn handle_variable(ctx: &Context, variable: &str) -> Result<i32, BotError> {
    ctx.db
        .get_user_variable(&ctx.room_id, &ctx.username, variable)
        .map_err(|e| match e {
            KeyDoesNotExist(_) => DiceRollingError::VariableNotFound(variable.to_owned()).into(),
            _ => e.into(),
        })
}

///Roll the dice in a dice pool, according to behavior documented in the various rolling
///methods.
fn roll_dice<R: DieRoller>(
    pool: &DicePoolWithContext,
    roller: &mut R,
) -> Result<RolledDicePool, BotError> {
    if pool.0.amounts.len() > 100 {
        return Err(DiceRollingError::ExpressionTooLarge.into());
    }

    let num_dice = calculate_dice_amount(&pool)?;
    let rolls: Vec<i32> = (0..num_dice)
        .flat_map(|_| roll_die(roller, &pool.0))
        .collect();
    Ok(RolledDicePool::from(&pool.0, num_dice, rolls))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use tempfile::tempdir;

    ///Instead of being random, generate a series of numbers we have complete
    ///control over.
    struct SequentialDieRoller {
        results: Vec<i32>,
        position: usize,
    }

    impl SequentialDieRoller {
        fn new(results: Vec<i32>) -> SequentialDieRoller {
            SequentialDieRoller {
                results: results,
                position: 0,
            }
        }
    }

    impl DieRoller for SequentialDieRoller {
        fn roll_number(&mut self, _sides: i32) -> i32 {
            let roll = self.results[self.position];
            self.position += 1;
            roll
        }
    }

    //Sanity checks
    #[test]
    pub fn chance_die_has_success_on_10_test() {
        assert_eq!(10, DicePool::chance_die().modifiers.success_on);
    }

    #[test]
    pub fn non_chance_die_has_success_on_8_test() {
        fn check_success_on(quality: DicePoolQuality) {
            let modifiers = DicePoolModifiers::custom_quality(quality);
            let amount = vec![Amount {
                operator: Operator::Plus,
                element: Element::Number(1),
            }];
            assert_eq!(8, DicePool::new(amount, modifiers).modifiers.success_on);
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
    fn dice_pool_modifiers_chance_die_test() {
        let modifiers = DicePoolModifiers::custom_quality(DicePoolQuality::ChanceDie);
        assert_eq!(10, modifiers.success_on);
    }

    #[test]
    fn dice_pool_modifiers_default_sanity_check() {
        let modifiers = DicePoolModifiers::default();
        assert_eq!(8, modifiers.success_on);
        assert_eq!(5, modifiers.exceptional_on);
        assert_eq!(DicePoolQuality::TenAgain, modifiers.quality);
    }

    #[test]
    pub fn no_explode_roll_test() {
        let db = Database::new(&sled::open(tempdir().unwrap()).unwrap());
        let ctx = Context::new(&db, "roomid", "username", "message");
        let pool = DicePool::easy_pool(1, DicePoolQuality::NoExplode);
        let pool_with_ctx = DicePoolWithContext(&pool, &ctx);

        let mut roller = SequentialDieRoller::new(vec![10, 8]);
        let result = roll_dice(&pool_with_ctx, &mut roller);
        assert!(result.is_ok());

        let roll = result.unwrap().roll;
        assert_eq!(vec![10], roll.rolls());
    }

    #[test]
    pub fn number_of_dice_equality_test() {
        let db = Database::new(&sled::open(tempdir().unwrap()).unwrap());
        let ctx = Context::new(&db, "roomid", "username", "message");
        let pool = DicePool::easy_pool(5, DicePoolQuality::NoExplode);
        let pool_with_ctx = DicePoolWithContext(&pool, &ctx);

        let mut roller = SequentialDieRoller::new(vec![1, 2, 3, 4, 5]);
        let result = roll_dice(&pool_with_ctx, &mut roller);
        assert!(result.is_ok());

        let roll = result.unwrap();
        assert_eq!(5, roll.num_dice);
    }

    #[test]
    fn rejects_large_expression_test() {
        let db = Database::new(&sled::open(tempdir().unwrap()).unwrap());
        let ctx = Context::new(&db, "roomid", "username", "message");

        let mut amounts = vec![];

        for _ in 0..500 {
            amounts.push(Amount {
                operator: Operator::Plus,
                element: Element::Number(1),
            });
        }

        let pool = DicePool::new(amounts, DicePoolModifiers::default());
        let pool_with_ctx = DicePoolWithContext(&pool, &ctx);

        let mut roller = SequentialDieRoller::new(vec![1, 2, 3, 4, 5]);
        let result = roll_dice(&pool_with_ctx, &mut roller);
        assert!(matches!(
            result,
            Err(BotError::DiceRollingError(
                DiceRollingError::ExpressionTooLarge
            ))
        ));
    }

    #[test]
    fn can_resolve_variables_test() {
        let db = Database::new(&sled::open(tempdir().unwrap()).unwrap());
        let ctx = Context::new(&db, "roomid", "username", "message");

        db.set_user_variable(&ctx.room_id, &ctx.username, "myvariable", 10)
            .expect("could not set myvariable to 10");

        let amounts = vec![Amount {
            operator: Operator::Plus,
            element: Element::Variable("myvariable".to_owned()),
        }];

        let pool = DicePool::new(amounts, DicePoolModifiers::default());
        let pool_with_ctx = DicePoolWithContext(&pool, &ctx);

        assert_eq!(calculate_dice_amount(&pool_with_ctx).unwrap(), 10);
    }

    //DicePool tests
    #[test]
    fn easy_pool_chance_die_test() {
        let pool = DicePool::easy_pool(1, DicePoolQuality::ChanceDie);
        assert_eq!(10, pool.modifiers.success_on);
    }

    #[test]
    fn easy_pool_quality_test() {
        fn check_quality(quality: DicePoolQuality) {
            let pool = DicePool::easy_pool(1, quality);
            assert_eq!(quality, pool.modifiers.quality);
        }

        check_quality(DicePoolQuality::TenAgain);
        check_quality(DicePoolQuality::NineAgain);
        check_quality(DicePoolQuality::EightAgain);
        check_quality(DicePoolQuality::Rote);
        check_quality(DicePoolQuality::ChanceDie);
        check_quality(DicePoolQuality::NoExplode);
    }

    #[test]
    fn is_successful_on_equal_test() {
        let result = DicePoolRoll {
            rolls: vec![8],
            modifiers: DicePoolModifiers {
                exceptional_on: 5,
                success_on: 8,
                quality: DicePoolQuality::TenAgain,
            },
        };

        assert_eq!(1, result.successes());
    }

    #[test]
    fn chance_die_success_test() {
        let result = DicePoolRoll {
            rolls: vec![10],
            modifiers: DicePoolModifiers {
                exceptional_on: 5,
                success_on: 10,
                quality: DicePoolQuality::ChanceDie,
            },
        };

        assert_eq!(1, result.successes());
    }

    #[test]
    fn chance_die_fail_test() {
        let result = DicePoolRoll {
            rolls: vec![9],
            modifiers: DicePoolModifiers {
                exceptional_on: 5,
                success_on: 10,
                quality: DicePoolQuality::ChanceDie,
            },
        };

        assert_eq!(0, result.successes());
    }

    #[test]
    fn is_exceptional_test() {
        let result = DicePoolRoll {
            rolls: vec![8, 8, 9, 10, 8],
            modifiers: DicePoolModifiers {
                exceptional_on: 5,
                success_on: 8,
                quality: DicePoolQuality::TenAgain,
            },
        };

        assert_eq!(5, result.successes());
        assert_eq!(true, result.is_exceptional());
    }

    #[test]
    fn is_not_exceptional_test() {
        let result = DicePoolRoll {
            rolls: vec![8, 8, 9, 10],
            modifiers: DicePoolModifiers::default(),
        };

        assert_eq!(4, result.successes());
        assert_eq!(false, result.is_exceptional());
    }

    //Format tests
    #[test]
    fn formats_dramatic_failure_test() {
        let result = DicePoolRoll {
            rolls: vec![1],
            modifiers: DicePoolModifiers::custom_quality(DicePoolQuality::ChanceDie),
        };

        assert_eq!("dramatic failure!", fmt_for_failure(&result));
    }

    #[test]
    fn formats_regular_failure_when_not_chance_die_test() {
        let result = DicePoolRoll {
            rolls: vec![1],
            modifiers: DicePoolModifiers {
                quality: DicePoolQuality::TenAgain,
                exceptional_on: 5,
                success_on: 10,
            },
        };

        assert_eq!("failure!", fmt_for_failure(&result));
    }

    #[test]
    fn formats_lots_of_dice_test() {
        let result = DicePoolRoll {
            modifiers: DicePoolModifiers::default(),
            rolls: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        };

        assert_eq!(
            "1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 1, 2, 3, 4, 5, and 4 more",
            fmt_rolls(&result)
        );
    }
}
